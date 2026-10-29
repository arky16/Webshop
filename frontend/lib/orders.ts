import api from "@/lib/api";
import type {CheckoutResponse, OrderWithItems} from "@/types/order";

export async function checkout(): Promise<CheckoutResponse> {
    const response = await api.post<CheckoutResponse>("/orders/checkout");

    return response.data;
}

export async function getMyOrders(): Promise<OrderWithItems[]> {
    const response = await api.get<OrderWithItems[]>("/orders");

    return response.data;
}

export async function getOrderDetails(orderId: string): Promise<OrderWithItems> {
    const response = await api.get<OrderWithItems>(`/orders/${orderId}`);

    return response.data;
}