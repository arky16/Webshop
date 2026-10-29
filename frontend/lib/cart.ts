import api from "@/lib/api";
import type {CartItem, AddCartItemRequest} from "@/types/cart";

function notifyCartUpdated() {
    if (typeof window !== "undefined") {
        window.dispatchEvent(new Event("cart-updated"));
    }
}

export async function getCart(): Promise<CartItem[]> {
    const response = await api.get<CartItem[]>("/cart");
    return response.data;
}

export async function addToCart(productId: string, quantity: number = 1): Promise<CartItem> {
    const body: AddCartItemRequest = {
        product_id: productId,
        quantity,
    };

    const response = await api.post<CartItem>("/cart", body);

    notifyCartUpdated();

    return response.data;
}

export async function removeFromCart(productId: string): Promise<void> {
    await api.delete(`/cart/${productId}`);

    notifyCartUpdated();
}
