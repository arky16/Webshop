import { describe, it, expect, beforeEach, vi } from "vitest";
import { getCart, addToCart, removeFromCart } from "@/lib/cart";
import api from "@/lib/api";
import type { CartItem } from "@/types/cart";

vi.mock("@/lib/api", () => ({
    default: {
        get: vi.fn(),
        post: vi.fn(),
        delete: vi.fn(),
    },
}));

const mockCartItem: CartItem = {
    id: "cart-item-123",
    product_id: "product-123",
    name: "Mechanische Tastatur",
    description: "Mechanische Gaming-Tastatur",
    price: "79.99",
    stock: 8,
    quantity: 2,
    subtotal: "159.98",
};


describe("Cart Service", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("sollte den Warenkorb vom Backend laden", async () => {
        vi.mocked(api.get).mockResolvedValue({
            data: [mockCartItem],
        });

        const result = await getCart();

        expect(api.get).toHaveBeenCalledWith("/cart");
        expect(result).toEqual([mockCartItem]);
    });

    it("sollte ein Produkt zum Warenkorb hinzufügen", async () => {
        vi.mocked(api.post).mockResolvedValue({
            data: mockCartItem,
        });

        const eventListener = vi.fn();
        window.addEventListener("cart-updated", eventListener);

        const result = await addToCart("product-123", 2);

        expect(api.post).toHaveBeenCalledWith("/cart", {
            product_id: "product-123",
            quantity: 2,
        });

        expect(result).toEqual(mockCartItem);
        expect(eventListener).toHaveBeenCalledTimes(1);

        window.removeEventListener("cart-updated", eventListener);
    });

    it("sollte standardmäßig die Menge 1 verwenden", async () => {
        vi.mocked(api.post).mockResolvedValue({
            data: mockCartItem,
        });

        await addToCart("product-123");

        expect(api.post).toHaveBeenCalledWith("/cart", {
            product_id: "product-123",
            quantity: 1,
        });
    });

    it("sollte ein Produkt aus dem Warenkorb entfernen", async () => {
        vi.mocked(api.delete).mockResolvedValue({
            data: undefined,
        });

        const eventListener = vi.fn();
        window.addEventListener("cart-updated", eventListener);

        await removeFromCart("product-123");

        expect(api.delete).toHaveBeenCalledWith("/cart/product-123");
        expect(eventListener).toHaveBeenCalledTimes(1);

        window.removeEventListener("cart-updated", eventListener);
    });

    it("sollte bei einem API-Fehler kein cart-updated Event senden", async () => {
        vi.mocked(api.post).mockRejectedValue(new Error("Server error"));

        const eventListener = vi.fn();
        window.addEventListener("cart-updated", eventListener);

        await expect(addToCart("product-123", 1)).rejects.toThrow(
            "Server error"
        );

        expect(eventListener).not.toHaveBeenCalled();

        window.removeEventListener("cart-updated", eventListener);
    });
});
