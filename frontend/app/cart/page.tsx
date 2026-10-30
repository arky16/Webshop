"use client"

import {useCallback, useEffect, useState} from "react";
import Link from "next/link";
import {useAuth} from "@/context/AuthContext";
import {getCart, removeFromCart} from "@/lib/cart";
import type {CartItem} from "@/types/cart";
import {useRouter} from "next/navigation";

export default function CartPage(){
    const {status} = useAuth();

    const [items, setItems] = useState<CartItem[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState("");
    const [removingId, setRemovingID] = useState<string | null>(null);
    const router = useRouter();

    const loadCart = useCallback(async () => {
        setLoading(true);
        setError("");

        try {
            const data =await getCart();
            setItems(data);
        } catch {
            setError("Der Warenkorb konnte nicht geladen werden");
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        if (status !== "authenticated") {
            return;
        }

        let cancelled = false;

        async function fetchCart() {
            setLoading(true);
            setError("");

            try {
                const data = await getCart();

                if (!cancelled) {
                    setItems(data);
                }
            } catch {
                if (!cancelled) {
                    setError("Der Warenkorb konnte nicht geladen werden");
                }
            } finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        }

        void fetchCart();

        return () => {
            cancelled = true;
        };
    }, [status]);

    async function handleRemove(productId: string) {
        setRemovingID(productId);
        setError("");

        try {
            await removeFromCart(productId);
            setItems((current) => current.filter((item) => item.product_id !== productId));
        } catch {
            setError("Das Product konnte nicht entfernt werden");
        } finally {
            setRemovingID(null);
        }
    }

    const total = items.reduce(
        (sum, item) =>
            sum + Number(item.subtotal),
        0
    );

    if (status === "loading") {
        return <p className="p-6">Authentifizierung wird geprüft...</p>;
    }

    if (status === "anonymous"){
        return (
            <main className="mx-auto max-w-4xl p-6">
                <h1 className="mb-4 text-3xl font-bold">Mein Warenkorb</h1>
                <p className="mb-4">Bitte melde dich an, um deinen Warenkorb zu sehen.</p>
                <Link
                    href="/login"
                    className="inline-block rounded-lg bg-gray-900 px-5 py-3 text-white hover:bg-blue-700"
                >
                    Zum Login
                </Link>
            </main>
        );
    }

    return (
        <main className="mx-auto max-w-4xl p-6">
            <h1 className="mb-6 text-3xl font-bold">Mein Warenkorb</h1>

            {loading ? (
                <p role="status">Warenkorb wird geladen...</p>
            ) : error && items.length === 0 ? (
                <div>
                    <p role="alert" className="mb-4 text-red-600">{error}</p>
                    <button
                        onClick={() => void loadCart()}
                        className="rounded-lg border px-4 py-2"
                    >
                        Erneut versuchen
                    </button>
                </div>
            ) : items.length === 0 ? (
                <div>
                    <p className="mb-4 text-gray-50">Dein Warenkorb ist noch leer.</p>
                    <Link
                        href="/"
                        className="inline-block rounded-lg bg-gray-900 px-5 py-3 text-white hover:bg-blue-700"
                    >
                        Produkte entdecken
                    </Link>
                </div>
            ) : (
                <>
                    {error && (
                        <p role="alert" className="mb-4 text-red-600">{error}</p>
                    )}

                    <div className="divide-y rounded-xl border">
                        {items.map((item) => (
                            <article
                                key={item.id}
                                className="flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between bg-gray-900"
                            >
                                <div className="min-w-0">
                                    <Link
                                        href={`/products/${item.product_id}`}
                                        className="font-semibold text-gray-100 hover:text-blue-700"
                                    >
                                        {item.name}
                                    </Link>

                                    <p className="mt-1 text-sm text-gray-600">
                                        Einzelpreis: {Number(item.price).toFixed(2)} €
                                    </p>

                                    <p className="text-sm text-gray-600">
                                        Menge: {item.quantity}
                                    </p>

                                    <p className="mt-2 font-medium">
                                        Zwischensumme: {Number(item.subtotal).toFixed(2)} €
                                    </p>
                                </div>

                                <button
                                    type="button"
                                    disabled={removingId !== null}
                                    onClick={() => void handleRemove(item.product_id)}
                                    className="self-start rounded-lg border border-red-200 px-4 py-2 text-sm text-red-700 hover:bg-red-50 disabled:opacity-50 sm:self-center"
                                >
                                    {removingId === item.product_id
                                        ? "Wird entfernt..."
                                        : "Entfernen"}
                                </button>
                            </article>
                        ))}
                    </div>

                    <div className="mt-6 flex flex-col items-end gap-3">
                        <p className="text-xl font-bold">
                            Gesamt: {total.toFixed(2)} €
                        </p>

                        <p className="text-xs text-gray-500">
                            Versandkosten und endgültiger Gesamtbetrag werden beim Checkout geprüft.
                        </p>

                        <button
                            type="button"
                            onClick={() => router.push("/checkout")}
                            className="rounded bg-blue-700 px-6 py-3 text-white"
                        >
                            Zur Kasse
                        </button>
                    </div>
                </>
            )}
        </main>
    );
}