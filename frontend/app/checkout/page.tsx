"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { getCart } from "@/lib/cart";
import { checkout } from "@/lib/orders";
import { useAuth } from "@/context/AuthContext";
import type { CartItem } from "@/types/cart";
import axios from "axios";

function formatPrice(price: string): string {
    return new Intl.NumberFormat("de-DE", {
        style: "currency",
        currency: "EUR",
    }).format(Number(price));
}

export default function CheckoutPage() {
    const router = useRouter();
    const { status } = useAuth();
    const [items, setItems] = useState<CartItem[]>([]);
    const [loading, setLoading] = useState(true);
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState("");

    useEffect(() => {
        if (status === "loading") return;

        if (status === "anonymous") {
            router.replace("/login");
            return;
        }

        let cancelled = false;

        async function loadCart() {
            try {
                setLoading(true);
                setError("");

                const cart = await getCart();

                if (!cancelled) {
                    setItems(cart);
                }
            } catch {
                if (!cancelled) {
                    setError("Dein Warenkorb konnte nicht geladen werden.");
                }
            } finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        }

        void loadCart();

        return () => {
            cancelled = true;
        };
    }, [status, router]);

    const total = items.reduce((sum, item) => sum + Number(item.subtotal), 0);

    async function handleCheckout() {
        if (submitting || items.length === 0) return;

        try {
            setSubmitting(true);
            setError("");

            const result = await checkout();

            router.push(`/orders/success?orderId=${encodeURIComponent(result.order.id)}`);
        } catch (err: unknown) {
            if (axios.isAxiosError(err)) {
                const code: unknown = err.response?.data?.error;

                if (err.response?.status === 401) {
                    router.replace("/login");
                    return;
                }

                if (code === "EMPTY_CART") {
                    setError("Dein Warenkorb ist leer. Bitte lade die Seite erneut.");
                    return;
                }

                if (code === "INSUFFICIENT_STOCK") {
                    setError("Ein Produkt ist nicht mehr verfügbar oder der Lagerbestand reicht nicht aus. Bitte überprüfe deinen Warenkorb.");
                    return;
                }
            }

            setError("Die Bestellung konnte nicht abgeschlossen werden. Bitte versuche es erneut.");
        } finally {
            setSubmitting(false);
        }
    }

    if (status === "loading" || loading) {
        return (
            <main className="mx-auto max-w-4xl p-6">
                <p role="status">Warenkorb wird geladen...</p>
            </main>
        );
    }

    if (status === "anonymous") {
        return (
            <main className="mx-auto max-w-4xl p-6">
                <p>Weiterleitung zum Login...</p>
            </main>
        );
    }

    return (
        <main className="mx-auto max-w-4xl p-6">
            <h1 className="mb-6 text-3xl font-bold">
                Bestellung überprüfen
            </h1>

            {error && (
                <div
                    role="alert"
                    className="mb-6 rounded border border-red-300 bg-red-50 p-4 text-red-800"
                >
                    {error}
                </div>
            )}

            {items.length === 0 ? (
                <div className="rounded border p-6">
                    <p className="mb-4">Dein Warenkorb ist leer.</p>

                    <button
                        type="button"
                        onClick={() => router.push("/")}
                        className="rounded bg-black px-4 py-2 text-white"
                    >
                        Weiter einkaufen
                    </button>
                </div>
            ) : (
                <>
                    <section
                        aria-labelledby="order-summary"
                        className="rounded-lg border p-6"
                    >
                        <h2
                            id="order-summary"
                            className="mb-4 text-xl font-semibold"
                        >
                            Deine Bestellübersicht
                        </h2>

                        <ul className="divide-y">
                            {items.map((item) => (
                                <li
                                    key={item.id}
                                    className="flex flex-wrap items-start justify-between gap-4 py-4"
                                >
                                    <div>
                                        <h3 className="font-medium">{item.name}</h3>

                                        <p className="text-sm text-gray-600">
                                            Menge: {item.quantity}
                                        </p>

                                        <p className="text-sm text-gray-600">
                                            Einzelpreis: {formatPrice(item.price)}
                                        </p>
                                    </div>

                                    <p className="font-semibold">
                                        {formatPrice(item.subtotal)}
                                    </p>
                                </li>
                            ))}
                        </ul>

                        <div className="mt-6 flex justify-between border-t pt-4 text-lg font-bold">
                            <span>Gesamtsumme</span>
                            <span>{formatPrice(String(total))}</span>
                        </div>

                        <p className="mt-2 text-sm text-gray-500">
                            Die endgültige Bestellsumme wird vom Server berechnet.
                        </p>
                    </section>

                    <div className="mt-6 flex flex-col gap-3 sm:flex-row sm:justify-between">
                        <button
                            type="button"
                            disabled={submitting}
                            onClick={() => router.push("/cart")}
                            className="rounded bg-gray-600 border px-5 py-3 disabled:opacity-50"
                        >
                            Zurück zum Warenkorb
                        </button>

                        <button
                            type="button"
                            disabled={submitting || items.length === 0}
                            onClick={handleCheckout}
                            className="rounded bg-blue-700 px-6 py-3 font-medium text-white disabled:cursor-not-allowed disabled:opacity-50"
                        >
                            {submitting
                                ? "Bestellung wird verarbeitet..."
                                : "Kostenpflichtig bestellen"}
                        </button>
                    </div>
                </>
            )}
        </main>
    );
}
