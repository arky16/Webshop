"use client"

import {createContext, useCallback, useContext, useEffect, useState, type ReactNode} from "react";
import {useAuth} from "@/context/AuthContext";
import {getCart} from "@/lib/cart";

interface CartContextType {
    itemCount: number;
    refreshCartCount: () => Promise<void>;
}

const CartContext = createContext<CartContextType | undefined>(undefined);

export function CartProvider({children}: {children: ReactNode}){
    const {status} = useAuth();
    const [itemCount, setItemCount] = useState(0);

    const refreshCartCount = useCallback(async () => {
        try {
            const items = await getCart();

            const count = items.reduce((sum, item) => sum + item.quantity, 0);

            setItemCount(count);
        } catch {}
    }, []);

    useEffect(() => {
        if (status !== "authenticated") {
            return;
        }

        let cancelled = false;

        async function loadCartCount() {
            try {
                const items = await getCart();

                if (!cancelled) {
                    const count = items.reduce(
                        (sum, item) => sum + item.quantity,
                        0
                    );

                    setItemCount(count);
                }
            } catch {
            }
        }

        void loadCartCount();

        return () => {
            cancelled = true;
        };
    }, [status]);

    useEffect(() => {
        function handleCartUpdated() {
            void refreshCartCount();
        }

        window.addEventListener("cart-updated", handleCartUpdated);

        return () => {
            window.removeEventListener("cart-updated", handleCartUpdated);
        };
    }, [refreshCartCount]);

    return (
        <CartContext.Provider
            value={{
                itemCount: status === "anonymous" ? 0 : itemCount,
                refreshCartCount,
            }}
        >
            {children}
        </CartContext.Provider>
    );
}

export function useCartCount(){
    const context = useContext(CartContext);

    if (!context) {
        throw new Error("useCartCount must be used inside CartProvider")
    }

    return context;
}