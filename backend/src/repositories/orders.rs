use crate::models::order::{Order, OrderItem};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub enum CheckoutError{
    EmptyCart,
    InsufficientStock,
    Database(sqlx::Error)
}

#[derive(Debug)]
pub enum UpdateOrderStatusError{
    NotFound,
    InvalidTransition,
    Database(sqlx::Error)
}

pub async fn checkout(pool: &PgPool, user_id: Uuid) -> Result<(Order, Vec<OrderItem>), CheckoutError>{
    let mut tx = pool.begin().await.map_err(CheckoutError::Database)?;

    let cart_items = sqlx::query_as::<_, (Uuid, i32)>(
        r#"
            SELECT product_id, quantity
            FROM cart_items
            WHERE user_id = $1
            ORDER BY product_id
            FOR UPDATE
            "#,
    )
        .bind(user_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(CheckoutError::Database)?;

    if cart_items.is_empty(){
        return Err(CheckoutError::EmptyCart);
    }

    let order_id = Uuid::new_v4();
    let mut total = Decimal::ZERO;
    let mut order_items = Vec::new();

    for (product_id, quantity) in cart_items{
        let product = sqlx::query_as::<_, (Decimal, i32, bool)>(
            r#"
                SELECT price, stock, active
                FROM products
                WHERE id = $1
                FOR UPDATE
                "#,
        )
            .bind(product_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(CheckoutError::Database)?;

        let (price, stock, active) = product.ok_or(CheckoutError::InsufficientStock)?;

        if !active || stock < quantity {
            return Err(CheckoutError::InsufficientStock);
        }

        let updated = sqlx::query(
            r#"
                UPDATE products
                SET stock = stock - $1,
                    updated_at = NOW()
                WHERE id = $2 AND active = TRUE AND stock >= $1
                "#,
        )
            .bind(quantity)
            .bind(product_id)
            .execute(&mut *tx)
            .await
            .map_err(CheckoutError::Database)?;

        if updated.rows_affected() != 1 {
            return Err(CheckoutError::InsufficientStock)
        }

        total += price * Decimal::from(quantity);

        order_items.push(OrderItem{
            id: Uuid::new_v4(),
            order_id,
            product_id,
            quantity,
            unit_price: price,
        });
    }

    let order = sqlx::query_as::<_, Order>(
        r#"
            INSERT INTO orders (id, user_id, status, total_amount)
            VALUES ($1, $2, 'PENDING', $3)
            RETURNING id, user_id, status, total_amount, created_at, updated_at
            "#,
    )
        .bind(order_id)
        .bind(user_id)
        .bind(total)
        .fetch_one(&mut *tx)
        .await
        .map_err(CheckoutError::Database)?;

    for item in &order_items{
        sqlx::query(
            r#"
                INSERT INTO order_items (id, order_id, product_id, quantity, unit_price)
                VALUES ($1, $2, $3, $4, $5)
                "#,
        )
            .bind(item.id)
            .bind(item.order_id)
            .bind(item.product_id)
            .bind(item.quantity)
            .bind(item.unit_price)
            .execute(&mut *tx)
            .await
            .map_err(CheckoutError::Database)?;
    }

    sqlx::query("DELETE FROM cart_items WHERE user_id =$1")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(CheckoutError::Database)?;

    tx.commit().await.map_err(CheckoutError::Database)?;

    Ok((order, order_items))
}

pub async fn find_orders_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Order>, sqlx::Error>{
    sqlx::query_as::<_, Order>(
        r#"
            SELECT id, user_id, status, total_amount, created_at, updated_at
            FROM orders
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#
    )
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub async fn find_order_items(pool: &PgPool, order_id: Uuid) -> Result<Vec<OrderItem>, sqlx::Error>{
    sqlx::query_as::<_, OrderItem>(
        r#"
            SELECT id, order_id, product_id, quantity, unit_price
            FROM order_items
            WHERE order_id = $1
            ORDER BY id
            "#
    )
        .bind(order_id)
        .fetch_all(pool)
        .await
}

pub async fn find_all_orders(pool: &PgPool, page: i64, size: i64, status: Option<&str>) -> Result<(Vec<Order>, i64), sqlx::Error>{
    let offset = page * size;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM orders
        WHERE ($1::text IS NULL OR status = $1)
        "#
    )
        .bind(status)
        .fetch_one(pool)
        .await?;

    let orders = sqlx::query_as::<_, Order>(
        r#"
        SELECT id, user_id, status, total_amount,
               created_at, updated_at
        FROM orders
        WHERE ($1::text IS NULL OR status = $1)
        ORDER BY created_at DESC, id DESC
        LIMIT $2 OFFSET $3
        "#
    )
        .bind(status)
        .bind(size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok((orders, total))
}

fn is_valid_order_status_transition(current_status: &str, new_status: &str) -> bool {
    matches!(
        (current_status, new_status),
        ("PENDING", "PAID")
            | ("PENDING", "PROCESSING")
            | ("PENDING", "CANCELLED")
            | ("PAID", "PROCESSING")
            | ("PROCESSING", "SHIPPED")
            | ("SHIPPED", "COMPLETED")
    )
}

pub async fn update_order_status(pool: &PgPool, order_id: Uuid, new_status: &str) -> Result<Order, UpdateOrderStatusError>{
    let mut tx = pool.begin().await.map_err(UpdateOrderStatusError::Database)?;

    let current_status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM orders WHERE id = $1 FOR UPDATE"
    )
        .bind(order_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(UpdateOrderStatusError::Database)?;

    let current_status = match current_status {
        Some(status) => status,
        None => return Err(UpdateOrderStatusError::NotFound)
    };

    if !is_valid_order_status_transition(
        current_status.as_str(),
        new_status,
    ) {
        return Err(UpdateOrderStatusError::InvalidTransition);
    }

    if new_status == "CANCELLED" {
        sqlx::query(
            r#"
                UPDATE products p
                SET stock = p.stock + oi.quantity, updated_at = NOW()
                FROM order_items oi
                WHERE oi.order_id = $1 AND oi.product_id = p.id
                "#
        )
            .bind(order_id)
            .execute(&mut *tx)
            .await
            .map_err(UpdateOrderStatusError::Database)?;
    }

    let order = sqlx::query_as::<_, Order>(
        r#"
            UPDATE orders
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, user_id, status, total_amount, created_at, updated_at
            "#
    )
        .bind(order_id)
        .bind(new_status)
        .fetch_one(&mut *tx)
        .await
        .map_err(UpdateOrderStatusError::Database)?;

    tx.commit().await.map_err(UpdateOrderStatusError::Database)?;

    Ok(order)
}

pub async fn find_order_items_for_orders(pool: &PgPool, order_ids: &[Uuid]) -> Result<Vec<OrderItem>, sqlx::Error>{
    sqlx::query_as::<_, OrderItem>(
        r#"
            SELECT id, order_id, product_id, quantity, unit_price
            FROM order_items
            WHERE order_id = ANY($1)
            ORDER BY order_id, id
            "#
    )
        .bind(order_ids)
        .fetch_all(pool)
        .await
}

pub async fn find_order_by_user(pool: &PgPool, order_id: Uuid, user_id: Uuid) -> Result<Option<Order>, sqlx::Error>{
    sqlx::query_as::<_, Order>(
        r#"
            SELECT id, user_id, status, total_amount, created_at, updated_at
            FROM orders
            WHERE id = $1 AND user_id = $2
            "#
    )
        .bind(order_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

#[cfg(test)]
mod tests {
    use super::is_valid_order_status_transition;

    #[test]
    fn pending_to_paid_is_valid() {
        assert!(is_valid_order_status_transition("PENDING", "PAID"));
    }

    #[test]
    fn pending_to_cancelled_is_valid() {
        assert!(is_valid_order_status_transition("PENDING", "CANCELLED"));
    }

    #[test]
    fn paid_to_processing_is_valid() {
        assert!(is_valid_order_status_transition("PAID", "PROCESSING"));
    }

    #[test]
    fn processing_to_shipped_is_valid() {
        assert!(is_valid_order_status_transition("PROCESSING", "SHIPPED"));
    }

    #[test]
    fn shipped_to_completed_is_valid() {
        assert!(is_valid_order_status_transition("SHIPPED", "COMPLETED"));
    }

    #[test]
    fn completed_to_cancelled_is_invalid() {
        assert!(!is_valid_order_status_transition("COMPLETED", "CANCELLED"));
    }

    #[test]
    fn pending_to_completed_is_invalid() {
        assert!(!is_valid_order_status_transition("PENDING", "COMPLETED"));
    }

    #[test]
    fn unknown_status_is_invalid() {
        assert!(!is_valid_order_status_transition("UNKNOWN", "PAID"));
    }
}


#[cfg(test)]
mod checkout_integration_tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn checkout_repository_integration_test() {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL ist nicht gesetzt");

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Verbindung zur Testdatenbank fehlgeschlagen");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Migrationen fehlgeschlagen");

        let user_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let insufficient_product_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO users (id, email, password_hash)
             VALUES ($1, $2, 'test-hash')"
        )
            .bind(user_id)
            .bind(format!("checkout-{}@example.com", user_id))
            .execute(&pool)
            .await
            .expect("Testbenutzer konnte nicht angelegt werden");

        let empty_result = checkout(&pool, user_id).await;

        assert!(matches!(
            empty_result,
            Err(CheckoutError::EmptyCart)
        ));

        sqlx::query(
            "INSERT INTO products
                (id, name, description, price, stock, category, active)
             VALUES ($1, 'Checkout Test Product', 'Test', 19.99, 5, 'Test', TRUE)"
        )
            .bind(product_id)
            .execute(&pool)
            .await
            .expect("Testprodukt konnte nicht angelegt werden");

        sqlx::query(
            "INSERT INTO products
                (id, name, description, price, stock, category, active)
             VALUES ($1, 'Insufficient Stock Product', 'Test', 10.00, 1, 'Test', TRUE)"
        )
            .bind(insufficient_product_id)
            .execute(&pool)
            .await
            .expect("Zweites Testprodukt konnte nicht angelegt werden");

        sqlx::query(
            "INSERT INTO cart_items (id, user_id, product_id, quantity)
             VALUES ($1, $2, $3, 2)"
        )
            .bind(Uuid::new_v4())
            .bind(user_id)
            .bind(product_id)
            .execute(&pool)
            .await
            .expect("Warenkorbeintrag konnte nicht angelegt werden");

        let (order, items) = checkout(&pool, user_id)
            .await
            .expect("Checkout fehlgeschlagen");

        assert_eq!(order.user_id, user_id);
        assert_eq!(order.status, "PENDING");
        assert_eq!(order.total_amount, Decimal::new(3998, 2));

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].product_id, product_id);
        assert_eq!(items[0].quantity, 2);
        assert_eq!(items[0].unit_price, Decimal::new(1999, 2));

        let saved_items = find_order_items(&pool, order.id)
            .await
            .expect("Bestellpositionen konnten nicht geladen werden");

        assert_eq!(saved_items.len(), 1);
        assert_eq!(saved_items[0].quantity, 2);

        let stock: i32 = sqlx::query_scalar(
            "SELECT stock FROM products WHERE id = $1"
        )
            .bind(product_id)
            .fetch_one(&pool)
            .await
            .expect("Lagerbestand konnte nicht geladen werden");

        assert_eq!(stock, 3);

        let cart_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM cart_items WHERE user_id = $1"
        )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Warenkorb konnte nicht geprüft werden");

        assert_eq!(cart_count, 0);

        sqlx::query(
            "INSERT INTO cart_items (id, user_id, product_id, quantity)
             VALUES ($1, $2, $3, 2)"
        )
            .bind(Uuid::new_v4())
            .bind(user_id)
            .bind(insufficient_product_id)
            .execute(&pool)
            .await
            .expect("Test-Warenkorbeintrag konnte nicht angelegt werden");

        let failed_checkout = checkout(&pool, user_id).await;

        assert!(matches!(
            failed_checkout,
            Err(CheckoutError::InsufficientStock)
        ));

        let unchanged_stock: i32 = sqlx::query_scalar(
            "SELECT stock FROM products WHERE id = $1"
        )
            .bind(insufficient_product_id)
            .fetch_one(&pool)
            .await
            .expect("Lagerbestand konnte nicht geprüft werden");

        assert_eq!(unchanged_stock, 1);

        let order_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM orders WHERE user_id = $1"
        )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Bestellungen konnten nicht gezählt werden");

        assert_eq!(order_count, 1);

        let remaining_cart_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM cart_items WHERE user_id = $1"
        )
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("Warenkorb konnte nicht geprüft werden");

        assert_eq!(remaining_cart_count, 1);

        sqlx::query("DELETE FROM orders WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Bestellungen konnten nicht gelöscht werden");

        sqlx::query("DELETE FROM cart_items WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Warenkorb konnte nicht gelöscht werden");

        sqlx::query("DELETE FROM products WHERE id IN ($1, $2)")
            .bind(product_id)
            .bind(insufficient_product_id)
            .execute(&pool)
            .await
            .expect("Testprodukte konnten nicht gelöscht werden");

        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Testbenutzer konnte nicht gelöscht werden");

        pool.close().await;
    }
}


#[cfg(test)]
mod order_status_integration_tests {
    use super::*;
    use rust_decimal::Decimal;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn order_status_integration_test() {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL ist nicht gesetzt");

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Verbindung zur Testdatenbank fehlgeschlagen");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Migrationen fehlgeschlagen");

        let user_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let cancel_order_id = Uuid::new_v4();
        let progress_order_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO users (id, email, password_hash)
             VALUES ($1, $2, 'test-hash')"
        )
            .bind(user_id)
            .bind(format!("status-test-{}@example.com", user_id))
            .execute(&pool)
            .await
            .expect("Testbenutzer konnte nicht angelegt werden");

        sqlx::query(
            "INSERT INTO products
                (id, name, description, price, stock, category, active)
             VALUES ($1, 'Status Test Product', 'Test', 10.00, 3, 'Test', TRUE)"
        )
            .bind(product_id)
            .execute(&pool)
            .await
            .expect("Testprodukt konnte nicht angelegt werden");

        for order_id in [cancel_order_id, progress_order_id] {
            sqlx::query(
                "INSERT INTO orders (id, user_id, status, total_amount)
                 VALUES ($1, $2, 'PENDING', $3)"
            )
                .bind(order_id)
                .bind(user_id)
                .bind(Decimal::new(2000, 2))
                .execute(&pool)
                .await
                .expect("Testbestellung konnte nicht angelegt werden");
        }

        sqlx::query(
            "INSERT INTO order_items
                (id, order_id, product_id, quantity, unit_price)
             VALUES ($1, $2, $3, 2, 10.00)"
        )
            .bind(Uuid::new_v4())
            .bind(cancel_order_id)
            .bind(product_id)
            .execute(&pool)
            .await
            .expect("Bestellposition konnte nicht angelegt werden");

        let invalid_result = update_order_status(
            &pool,
            progress_order_id,
            "COMPLETED"
        ).await;

        assert!(matches!(
            invalid_result,
            Err(UpdateOrderStatusError::InvalidTransition)
        ));

        let paid_order = update_order_status(
            &pool,
            progress_order_id,
            "PAID"
        )
            .await
            .expect("Statuswechsel zu PAID fehlgeschlagen");

        assert_eq!(paid_order.status, "PAID");

        let processing_order = update_order_status(
            &pool,
            progress_order_id,
            "PROCESSING"
        )
            .await
            .expect("Statuswechsel zu PROCESSING fehlgeschlagen");

        assert_eq!(processing_order.status, "PROCESSING");

        let cancelled_order = update_order_status(
            &pool,
            cancel_order_id,
            "CANCELLED"
        )
            .await
            .expect("Stornierung fehlgeschlagen");

        assert_eq!(cancelled_order.status, "CANCELLED");

        let stock_after_cancellation: i32 = sqlx::query_scalar(
            "SELECT stock FROM products WHERE id = $1"
        )
            .bind(product_id)
            .fetch_one(&pool)
            .await
            .expect("Lagerbestand konnte nicht geladen werden");

        assert_eq!(stock_after_cancellation, 5);

        let second_cancellation = update_order_status(
            &pool,
            cancel_order_id,
            "CANCELLED"
        ).await;

        assert!(matches!(
            second_cancellation,
            Err(UpdateOrderStatusError::InvalidTransition)
        ));

        let stock_after_second_attempt: i32 = sqlx::query_scalar(
            "SELECT stock FROM products WHERE id = $1"
        )
            .bind(product_id)
            .fetch_one(&pool)
            .await
            .expect("Lagerbestand konnte nicht geladen werden");

        assert_eq!(stock_after_second_attempt, 5);

        let missing_order = update_order_status(
            &pool,
            Uuid::new_v4(),
            "PAID"
        ).await;

        assert!(matches!(
            missing_order,
            Err(UpdateOrderStatusError::NotFound)
        ));

        sqlx::query("DELETE FROM orders WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Testbestellungen konnten nicht gelöscht werden");

        sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(product_id)
            .execute(&pool)
            .await
            .expect("Testprodukt konnte nicht gelöscht werden");

        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("Testbenutzer konnte nicht gelöscht werden");

        pool.close().await;
    }
}

