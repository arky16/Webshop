"use client";

import {useEffect, useState, Suspense} from "react";
import {useSearchParams, useRouter} from "next/navigation";
import {getOrderDetails} from "@/lib/orders";
import type {OrderWithItems} from "@/types/order";
import {useAuth} from "@/context/AuthContext";

function formatPrice(price: string): string {
    return new Intl.NumberFormat("de-DE", {
        style: "currency",
        currency: "EUR",
    }).format(Number(price));
}

function OrderSuccessContent() {
    const searchParams = useSearchParams();
    const router = useRouter();
    const {status} = useAuth();

    const orderId = searchParams.get("orderId");

    const [result, setResult] = useState<OrderWithItems | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState("");

    useEffect(() => {
        if (status === "loading" || status === "anonymous" || !orderId) {
            return;
        }

        const currentOrderId = orderId;

        let cancelled = false;

        async function loadOrder() {
            try {
                setLoading(true);
                setError("");

                const data = await getOrderDetails(currentOrderId);

                if (!cancelled) {
                    setResult(data);
                }
            } catch {
                if (!cancelled) {
                    setError("Die Bestelldetails konnten nicht geladen werden.");
                }
            } finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        }

        void loadOrder();

        return () => {
            cancelled = true;
        };
    }, [status, orderId]);

    if (status === "loading" || loading) {
        return (
            <main className="mx-auto max-w-3xl p-6">
                <p role="status">Bestellung wird geladen...</p>
            </main>
        );
    }

    if (status === "anonymous") {
        return (
            <main className="mx-auto max-w-3xl p-6">
                <p>Weiterleitung zum Login...</p>
            </main>
        );
    }

    if (!orderId) {
        return (
            <main className="mx-auto max-w-3xl p-6">
                <div role="alert" className="rounded border p-6">
                    <h1 className="mb-3 text-2xl font-bold">
                        Bestelldetails nicht verfügbar
                    </h1>

                    <p className="mb-4 text-red-700">
                        Es wurde keine Bestellnummer angegeben.
                    </p>

                    <button
                        type="button"
                        onClick={() => router.push("/orders")}
                        className="rounded bg-black px-4 py-2 text-white"
                    >
                        Meine Bestellungen
                    </button>
                </div>
            </main>
        );
    }

    if (error || !result) {
        return (
            <main className="mx-auto max-w-3xl p-6">
                <div role="alert" className="rounded border p-6">
                    <h1 className="mb-3 text-2xl font-bold">
                        Bestelldetails nicht verfügbar
                    </h1>

                    <p className="mb-4 text-red-700">
                        {error || "Die Bestellung konnte nicht gefunden werden."}
                    </p>

                    <button
                        type="button"
                        onClick={() => router.push("/orders")}
                        className="rounded bg-black px-4 py-2 text-white"
                    >
                        Meine Bestellungen
                    </button>
                </div>
            </main>
        );
    }

    const { order, items } = result;

    return (
        <main className="mx-auto max-w-3xl p-6">
            <section className="mb-6 rounded-lg border p-6">
                <div className="mb-4 text-4xl" aria-hidden="true">
                    ✓
                </div>

                <h1 className="mb-3 text-3xl font-bold">
                    Vielen Dank für deine Bestellung!
                </h1>

                <p className="text-gray-600">
                    Deine Bestellung wurde erfolgreich erstellt.
                </p>

                <div className="mt-5 space-y-2">
                    <p>
                        <span className="font-semibold">Bestellnummer:</span>{" "}
                        <span className="break-all">{order.id}</span>
                    </p>

                    <p>
                        <span className="font-semibold">Bestellstatus:</span>{" "}
                        {order.status}
                    </p>

                    <p>
                        <span className="font-semibold">Bestelldatum:</span>{" "}
                        {new Date(order.created_at).toLocaleString("de-DE")}
                    </p>
                </div>
            </section>

            <section className="rounded-lg border p-6">
                <h2 className="mb-4 text-xl font-semibold">
                    Bestellübersicht
                </h2>

                <ul className="divide-y">
                    {items.map((item) => (
                        <li
                            key={item.id}
                            className="flex flex-wrap justify-between gap-4 py-4"
                        >
                            <div>
                                <p className="font-medium">
                                    Produkt-ID: {item.product_id}
                                </p>

                                <p className="text-sm text-gray-600">
                                    Menge: {item.quantity}
                                </p>

                                <p className="text-sm text-gray-600">
                                    Einzelpreis: {formatPrice(item.unit_price)}
                                </p>
                            </div>

                            <p className="font-semibold">
                                {formatPrice(String(Number(item.unit_price) * item.quantity))}
                            </p>
                        </li>
                    ))}
                </ul>

                <div className="mt-6 flex justify-between border-t pt-4 text-lg font-bold">
                    <span>Gesamtsumme</span>
                    <span>{formatPrice(order.total_amount)}</span>
                </div>
            </section>

            <div className="mt-6 flex flex-wrap justify-center gap-3">
                <button
                    type="button"
                    onClick={() => router.push("/orders")}
                    className="rounded bg-gray-600 px-5 py-3 text-white"
                >
                    Meine Bestellungen
                </button>

                <button
                    type="button"
                    onClick={() => router.push("/")}
                    className="rounded border bg-blue-700 px-5 py-3"
                >
                    Weiter einkaufen
                </button>
            </div>
        </main>
    );
}

export default function OrderSuccessPage() {
    return (
        <Suspense
            fallback={
                <main className="mx-auto max-w-3xl p-6">
                    <p role="status">
                        Bestelldetails werden geladen...
                    </p>
                </main>
            }
        >
            <OrderSuccessContent />
        </Suspense>
    );
}
