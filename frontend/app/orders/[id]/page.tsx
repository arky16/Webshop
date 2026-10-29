"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { getOrderDetails } from "@/lib/orders";
import type { OrderWithItems } from "@/types/order";
import { useAuth } from "@/context/AuthContext";
import axios from "axios";

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

export default function OrderDetailsPage() {
    const params = useParams<{ id: string }>();
    const router = useRouter();
    const { status } = useAuth();

    const orderId = params.id;

    const [result, setResult] = useState<OrderWithItems | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState("");

    useEffect(() => {
        if (status === "loading") return;

        if (status === "anonymous") {
            router.replace("/login");
            return;
        }

        let cancelled = false;

        async function loadOrder() {
            try {
                setLoading(true);
                setError("");

                const data = await getOrderDetails(orderId);

                if (!cancelled) {
                    setResult(data);
                }
            } catch (err: unknown) {
                if (!cancelled) {
                    if (axios.isAxiosError(err) && err.response?.status === 404) {
                        setError("Diese Bestellung wurde nicht gefunden.");
                    } else {
                        setError("Die Bestelldetails konnten nicht geladen werden.");
                    }
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
    }, [status, orderId, router]);

    if (status === "loading" || loading) {
        return (
            <main className="mx-auto max-w-4xl p-6">
                <p role="status">Bestelldetails werden geladen...</p>
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

    if (error || !result) {
        return (
            <main className="mx-auto max-w-4xl p-6">
                <div
                    role="alert"
                    className="rounded-lg border border-red-300 bg-red-50 p-6"
                >
                    <h1 className="mb-3 text-2xl text-black font-bold">
                        Bestellung nicht verfügbar
                    </h1>

                    <p className="mb-5 text-red-800">
                        {error || "Die Bestellung konnte nicht gefunden werden."}
                    </p>

                    <Link
                        href="/orders"
                        className="inline-block rounded bg-black px-5 py-3 text-white"
                    >
                        Zurück zu meinen Bestellungen
                    </Link>
                </div>
            </main>
        );
    }

    const { order, items } = result;

    return (
        <main className="mx-auto max-w-4xl p-6">
            <div className="mb-6">
                <Link
                    href="/orders"
                    className="text-sm text-gray-600 hover:underline"
                >
                    ← Zurück zu meinen Bestellungen
                </Link>

                <h1 className="mt-4 text-3xl font-bold">
                    Bestelldetails
                </h1>
            </div>

            <section className="mb-6 rounded-lg border p-6">
                <div className="flex flex-col justify-between gap-4 sm:flex-row">
                    <div>
                        <p className="text-sm text-gray-500">
                            Bestellnummer
                        </p>

                        <p className="break-all font-semibold">
                            {order.id}
                        </p>
                    </div>

                    <div>
                        <span className={`inline-block rounded-full px-3 py-1 text-sm font-medium ${getStatusClass(order.status)}`}>
                            {getStatusLabel(order.status)}
                        </span>
                    </div>
                </div>

                <div className="mt-6 grid gap-4 border-t pt-5 sm:grid-cols-2">
                    <div>
                        <p className="text-sm text-gray-500">
                            Bestelldatum
                        </p>
                        <p className="font-medium">
                            {formatDate(order.created_at)}
                        </p>
                    </div>

                    <div>
                        <p className="text-sm text-gray-500">
                            Letzte Aktualisierung
                        </p>
                        <p className="font-medium">
                            {formatDate(order.updated_at)}
                        </p>
                    </div>
                </div>
            </section>

            <section className="rounded-lg border p-6">
                <h2 className="mb-5 text-xl font-semibold">
                    Bestellte Produkte
                </h2>

                {items.length === 0 ? (
                    <p className="text-gray-600">
                        Diese Bestellung enthält keine Positionen.
                    </p>
                ) : (
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

                                    <p className="mt-1 text-sm text-gray-600">
                                        Menge: {item.quantity}
                                    </p>

                                    <p className="text-sm text-gray-600">
                                        Einzelpreis: {formatPrice(item.unit_price)}
                                    </p>
                                </div>

                                <div className="font-semibold">
                                    {formatPrice(
                                        String(
                                            Number(item.unit_price) * item.quantity
                                        )
                                    )}
                                </div>
                            </li>
                        ))}
                    </ul>
                )}

                <div className="mt-5 flex justify-between border-t pt-5 text-lg font-bold">
                    <span>Gesamtbetrag</span>
                    <span>{formatPrice(order.total_amount)}</span>
                </div>
            </section>

            <div className="mt-6">
                <Link
                    href="/orders"
                    className="inline-block rounded border px-5 py-3 hover:bg-blue-700"
                >
                    Zurück zur Bestellhistorie
                </Link>
            </div>
        </main>
    );
}
