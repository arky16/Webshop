export interface CartItem{
    id: string;
    product_id: string;
    name: string;
    description: string;
    price: string;
    stock: number;
    quantity: number;
    subtotal: string;
}

export interface AddCartItemRequest{
    product_id: string;
    quantity: number;
}