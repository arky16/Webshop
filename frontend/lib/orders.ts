import api from "@/lib/api";
import type {CheckoutResponse, OrderWithItems, AdminOrdersResponse, OrderStatus} from "@/types/order";

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

export async function getAllOrders(page: number = 0, size: number = 10, status?: OrderStatus): Promise<AdminOrdersResponse>{
    const response = await api.get<AdminOrdersResponse>("/admin/orders", {
        params: {
            page,
            size,
            ...(status? {status} : {}),
        }
    });

    return response.data;
}

export async function updateOrderStatus(orderId: string, status: OrderStatus): Promise<void>{
    await api.patch(`/admin/orders/${orderId}/status`, {status});
}