"use client";

import {useEffect, useState} from "react";
import Link from "next/link";
import {useRouter} from "next/navigation";
import {getMyOrders} from "@/lib/orders";
import type {OrderWithItems} from "@/types/order";
import {useAuth} from "@/context/AuthContext";

function formatPrice(price: string): string {
    return new Intl.NumberFormat("de-DE", {
        style: "currency",
        currency: "EUR",
    }).format(Number(price));
}

function formatDate(date: string): string {
    return new Date(date).toLocaleString("de-DE");
}

function getStatusLabel(status: string): string {
    const labels: Record<string, string> = {
        PENDING: "Ausstehend",
        PAID: "Bezahlt",
        PROCESSING: "In Bearbeitung",
        SHIPPED: "Versendet",
        COMPLETED: "Abgeschlossen",
        CANCELLED: "Storniert",
    };

    return labels[status] ?? status;
}

function getStatusClass(status: string): string {
    const classes: Record<string, string> = {
        PENDING: "bg-yellow-100 text-yellow-800",
        PAID: "bg-green-100 text-green-800",
        PROCESSING: "bg-blue-100 text-blue-800",
        SHIPPED: "bg-indigo-100 text-indigo-800",
        COMPLETED: "bg-green-100 text-green-800",
        CANCELLED: "bg-red-100 text-red-800",
    };

    return classes[status] ?? "bg-gray-100 text-gray-800";
}

export default function OrdersPage() {
    const router = useRouter();
    const { status } = useAuth();
    const [orders, setOrders] = useState<OrderWithItems[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState("");

    useEffect(() => {
        if (status === "loading") return;

        if (status === "anonymous") {
            router.replace("/login");
            return;
        }

        let cancelled = false;

        async function loadOrders() {
            try {
                setLoading(true);
                setError("");

                const data = await getMyOrders();

                if (!cancelled) {
                    setOrders(data);
                }
            } catch {
                if (!cancelled) {
                    setError("Deine Bestellungen konnten nicht geladen werden.");
                }
            } finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        }

        void loadOrders();

        return () => {
            cancelled = true;
        };
    }, [status, router]);

    if (status === "loading" || loading) {
        return (
            <main className="mx-auto max-w-5xl p-6">
                <p role="status">Bestellungen werden geladen...</p>
            </main>
        );
    }

    if (status === "anonymous") {
        return (
            <main className="mx-auto max-w-5xl p-6">
                <p>Weiterleitung zum Login...</p>
            </main>
        );
    }

    return (
        <main className="mx-auto max-w-5xl p-6">
            <div className="mb-8">
                <h1 className="text-3xl font-bold">
                    Meine Bestellungen
                </h1>

                <p className="mt-2 text-gray-50">
                    Hier findest du eine Übersicht deiner bisherigen Bestellungen.
                </p>
            </div>

            {error && (
                <div
                    role="alert"
                    className="mb-6 rounded border border-red-300 bg-red-50 p-4 text-red-800"
                >
                    {error}

                    <button
                        type="button"
                        onClick={() => router.refresh()}
                        className="ml-2 underline"
                    >
                        Erneut versuchen
                    </button>
                </div>
            )}

            {!error && orders.length === 0 && (
                <section className="rounded-lg border p-8 text-center">
                    <h2 className="text-xl font-semibold">
                        Noch keine Bestellungen
                    </h2>

                    <p className="mt-2 text-gray-600">
                        Sobald du etwas bestellst, erscheinen deine Bestellungen hier.
                    </p>

                    <button
                        type="button"
                        onClick={() => router.push("/")}
                        className="mt-5 rounded bg-black px-5 py-3 text-white"
                    >
                        Produkte entdecken
                    </button>
                </section>
            )}

            {orders.length > 0 && (
                <div className="space-y-5">
                    {orders.map(({ order, items }) => (
                        <article
                            key={order.id}
                            className="rounded-lg border p-5 bg-gray-900"
                        >
                            <div className="flex flex-col justify-between gap-4 sm:flex-row">
                                <div>
                                    <p className="text-sm text-gray-500">
                                        Bestellnummer
                                    </p>

                                    <p className="break-all font-semibold">
                                        {order.id}
                                    </p>

                                    <p className="mt-2 text-sm text-gray-600">
                                        Bestellt am: {formatDate(order.created_at)}
                                    </p>
                                </div>

                                <div className="sm:text-right">
                                    <span className={`inline-block rounded-full px-3 py-1 text-sm font-medium ${getStatusClass(order.status)}`}>
                                        {getStatusLabel(order.status)}
                                    </span>

                                    <p className="mt-3 text-xl font-bold">
                                        {formatPrice(order.total_amount)}
                                    </p>
                                </div>
                            </div>

                            <div className="mt-4 border-t pt-4">
                                <p className="mb-2 font-medium">
                                    {items.length}{" "}
                                    {items.length === 1
                                        ? "Bestellposition"
                                        : "Bestellpositionen"}
                                </p>

                                <ul className="space-y-1 text-sm text-gray-600">
                                    {items.map((item) => (
                                        <li key={item.id}>
                                            {item.quantity} × Produkt {item.product_id}
                                            {" — "}
                                            {formatPrice(item.unit_price)} pro Stück
                                        </li>
                                    ))}
                                </ul>
                            </div>

                            <div className="mt-5">
                                <Link
                                    href={`/orders/${encodeURIComponent(order.id)}`}
                                    className="inline-block rounded border px-4 py-2 font-medium hover:bg-blue-700"
                                >
                                    Details ansehen
                                </Link>
                            </div>
                        </article>
                    ))}
                </div>
            )}
        </main>
    );
}
