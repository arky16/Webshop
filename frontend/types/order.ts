export interface Order {
    id: string;
    user_id: string;
    status: string;
    total_amount: string;
    created_at: string;
    updated_at: string;
}

export interface OrderItem {
    id: string;
    order_id: string;
    product_id: string;
    quantity: number;
    unit_price: string;
}

export interface CheckoutResponse {
    order: Order;
    items: OrderItem[];
}

export interface OrderWithItems {
    order: Order;
    items: OrderItem[];
}
