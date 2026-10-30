use crate::models::cart::CartItemDetails;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn find_all(pool: &PgPool, user_id: Uuid) -> Result<Vec<CartItemDetails>, sqlx::Error>{
    sqlx::query_as::<_, CartItemDetails>(
        r#"
            SELECT 
                c.id,
                c.product_id,
                p.name,
                p.description,
                p.price,
                p.stock,
                c.quantity,
                (p.price * c.quantity)::NUMERIC(12,2) AS subtotal
            FROM cart_items c
            JOIN products p ON p.id = c.product_id
            WHERE c.user_id = $1 AND p.active = TRUE
            ORDER BY c.created_at DESC 
            "#
    )
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub async fn add_item(pool: &PgPool, user_id: Uuid, product_id: Uuid, quantity: i32) -> Result<Option<CartItemDetails>, sqlx::Error>{
    sqlx::query_as::<_, CartItemDetails>(
        r#"
            WITH upserted AS ( 
                INSERT INTO cart_items(id, user_id, product_id, quantity)
                SELECT $1, p_user.id, p.id, $4
                FROM products p 
                JOIN users p_user ON p_user.id =$2
                WHERE p.id =$3 AND p.active = TRUE AND p.stock >= $4
                ON CONFLICT (user_id, product_id)
                DO UPDATE SET quantity = cart_items.quantity + EXCLUDED.quantity, updated_at = NOW()
                WHERE cart_items.quantity + EXCLUDED.quantity <= (
                    SELECT stock
                    FROM products
                    WHERE id = EXCLUDED.product_id AND active = TRUE
                )
                RETURNING id ,user_id, product_id, quantity, created_at, updated_at
             )
            SELECT 
                c.id,
                c.product_id,
                p.name,
                p.description,
                p.price,
                p.stock,
                c.quantity,
                (p.price * c.quantity)::NUMERIC(12,2) AS subtotal
            FROM upserted c
            JOIN products p ON p.id = c.product_id
            "#
    )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(product_id)
        .bind(quantity)
        .fetch_optional(pool)
        .await
}

pub async fn remove_item(pool: &PgPool, user_id: Uuid, product_id: Uuid) -> Result<bool, sqlx::Error>{
    let result = sqlx::query(
        r#"
            DELETE FROM cart_items
            WHERE user_id = $1 AND product_id = $2
            "#
    )
        .bind(user_id)
        .bind(product_id)
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected() > 0)
}


#[cfg(test)]
mod integration_tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    #[tokio::test]
    async fn cart_repository_integration_test() {
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

        let user1_id = Uuid::new_v4();
        let user2_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        for (id, email) in [
            (user1_id, format!("test-{}@example.com", user1_id)),
            (user2_id, format!("test-{}@example.com", user2_id)),
        ] {
            sqlx::query(
                r#"
                INSERT INTO users (id, email, password_hash)
                VALUES ($1, $2, 'test-hash')
                "#
            )
                .bind(id)
                .bind(email)
                .execute(&pool)
                .await
                .expect("Testbenutzer konnte nicht angelegt werden");
        }

        sqlx::query(
            r#"
            INSERT INTO products
                (id, name, description, price, stock, category, active)
            VALUES
                ($1, 'Integration Test Product', 'Test description',
                 19.99, 5, 'Test', TRUE)
            "#
        )
            .bind(product_id)
            .execute(&pool)
            .await
            .expect("Testprodukt konnte nicht angelegt werden");

        let first_add = add_item(&pool, user1_id, product_id, 2)
            .await
            .expect("Erstes Hinzufügen fehlgeschlagen")
            .expect("Produkt wurde nicht hinzugefügt");

        assert_eq!(first_add.quantity, 2);

        let second_add = add_item(&pool, user1_id, product_id, 3)
            .await
            .expect("Zweites Hinzufügen fehlgeschlagen")
            .expect("Produktmenge wurde nicht erhöht");

        assert_eq!(second_add.quantity, 5);

        let overflow = add_item(&pool, user1_id, product_id, 1)
            .await
            .expect("Datenbankabfrage fehlgeschlagen");

        assert!(overflow.is_none());

        let user2_add = add_item(&pool, user2_id, product_id, 1)
            .await
            .expect("Hinzufügen für zweiten Benutzer fehlgeschlagen")
            .expect("Produkt für zweiten Benutzer nicht hinzugefügt");

        assert_eq!(user2_add.quantity, 1);

        let user1_cart = find_all(&pool, user1_id)
            .await
            .expect("Warenkorb von Benutzer 1 konnte nicht geladen werden");

        let user2_cart = find_all(&pool, user2_id)
            .await
            .expect("Warenkorb von Benutzer 2 konnte nicht geladen werden");

        assert_eq!(user1_cart.len(), 1);
        assert_eq!(user1_cart[0].quantity, 5);

        assert_eq!(user2_cart.len(), 1);
        assert_eq!(user2_cart[0].quantity, 1);

        let removed = remove_item(&pool, user1_id, product_id)
            .await
            .expect("Entfernen fehlgeschlagen");

        assert!(removed);

        let user1_cart_after_removal = find_all(&pool, user1_id)
            .await
            .expect("Warenkorb konnte nicht erneut geladen werden");

        assert!(user1_cart_after_removal.is_empty());

        let user2_cart_after_removal = find_all(&pool, user2_id)
            .await
            .expect("Warenkorb von User 2 konnte nicht geladen werden");

        assert_eq!(user2_cart_after_removal.len(), 1);

        sqlx::query("DELETE FROM cart_items WHERE user_id IN ($1, $2)")
            .bind(user1_id)
            .bind(user2_id)
            .execute(&pool)
            .await
            .expect("Cart-Testdaten konnten nicht gelöscht werden");

        sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(product_id)
            .execute(&pool)
            .await
            .expect("Testprodukt konnte nicht gelöscht werden");

        sqlx::query("DELETE FROM users WHERE id IN ($1, $2)")
            .bind(user1_id)
            .bind(user2_id)
            .execute(&pool)
            .await
            .expect("Testbenutzer konnten nicht gelöscht werden");

        pool.close().await;
    }
}
