"use client";

import {useCallback, useEffect, useState} from "react";
import {useRouter} from "next/navigation";
import Link from "next/link";
import {useAuth} from "@/context/AuthContext";
import {getAllOrders, updateOrderStatus} from "@/lib/orders";
import type {OrderWithItems, OrderStatus} from "@/types/order";

const statuses: OrderStatus[] = [
    "PENDING",
    "PAID",
    "PROCESSING",
    "SHIPPED",
    "COMPLETED",
    "CANCELLED",
];

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

function formatPrice(price: string): string {
    return new Intl.NumberFormat("de-DE", {
        style: "currency",
        currency: "EUR",
    }).format(Number(price));
}

export default function AdminOrdersPage() {
    const router = useRouter();
    const { status, user } = useAuth();
    const [orders, setOrders] = useState<OrderWithItems[]>([]);
    const [page, setPage] = useState(0);
    const [totalPages, setTotalPages] = useState(0);
    const [total, setTotal] = useState(0);
    const [filterStatus, setFilterStatus] = useState("");
    const [loading, setLoading] = useState(true);
    const [updatingId, setUpdatingId] = useState<string | null>(null);
    const [error, setError] = useState("");
    const [success, setSuccess] = useState("");
    const isAdmin = user?.role === "ADMIN";

    const loadOrders = useCallback(async () => {
        try {
            setLoading(true);
            setError("");

            const data = await getAllOrders(
                page,
                10,
                filterStatus ? (filterStatus as OrderStatus) : undefined
            );

            setOrders(data.orders);
            setTotal(data.pagination.total);
            setTotalPages(data.pagination.total_pages);
        } catch {
            setError("Bestellungen konnten nicht geladen werden.");
        } finally {
            setLoading(false);
        }
    }, [page, filterStatus]);

    useEffect(() => {
        if (status === "loading") return;

        if (status === "anonymous") {
            router.replace("/login");
            return;
        }

        if (user && !isAdmin) {
            router.replace("/");
        }
    }, [status, user, isAdmin, router]);

    useEffect(() => {
        if (status !== "authenticated" || !isAdmin) {
            return;
        }

        let cancelled = false;

        async function fetchOrders() {
            try {
                setLoading(true);
                setError("");

                const data = await getAllOrders(
                    page,
                    10,
                    filterStatus ? (filterStatus as OrderStatus) : undefined
                );

                if (cancelled) return;

                setOrders(data.orders);
                setTotal(data.pagination.total);
                setTotalPages(data.pagination.total_pages);
            } catch {
                if (!cancelled) {
                    setError("Bestellungen konnten nicht geladen werden.");
                }
            } finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        }

        void fetchOrders();

        return () => {
            cancelled = true;
        };
    }, [status, isAdmin, page, filterStatus]);

    async function handleStatusChange(
        orderId: string,
        newStatus: OrderStatus
    ) {
        try {
            setUpdatingId(orderId);
            setError("");
            setSuccess("");

            await updateOrderStatus(orderId, newStatus);

            setSuccess("Bestellstatus erfolgreich aktualisiert.");

            await loadOrders();
        } catch {
            setError("Der Bestellstatus konnte nicht geändert werden. Möglicherweise ist dieser Statuswechsel nicht erlaubt.");
        } finally {
            setUpdatingId(null);
        }
    }

    if (status === "loading" || loading && status !== "authenticated") {
        return (
            <main className="mx-auto max-w-6xl p-6">
                <p role="status">Admin-Bereich wird geladen...</p>
            </main>
        );
    }

    if (status === "anonymous" || !isAdmin) {
        return (
            <main className="mx-auto max-w-6xl p-6">
                <p>Weiterleitung...</p>
            </main>
        );
    }

    return (
        <main className="mx-auto max-w-6xl p-6">
            <div className="mb-8">
                <h1 className="text-3xl font-bold">
                    Bestellverwaltung
                </h1>

                <p className="mt-2 text-gray-50">
                    Verwalte alle Bestellungen deiner Kunden.
                </p>
            </div>

            {error && (
                <div
                    role="alert"
                    className="mb-4 rounded border border-red-300 bg-red-50 p-4 text-red-800"
                >
                    {error}
                </div>
            )}

            {success && (
                <div
                    role="status"
                    className="mb-4 rounded border border-green-300 bg-green-50 p-4 text-green-800"
                >
                    {success}
                </div>
            )}

            <section className="mb-6 rounded-lg border p-4 bg-gray-900">
                <div className="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
                    <div>
                        <p className="text-sm text-gray-500">
                            Gefundene Bestellungen
                        </p>

                        <p className="text-2xl font-bold">{total}</p>
                    </div>

                    <div>
                        <label
                            htmlFor="status-filter"
                            className="mb-1 block text-sm font-medium"
                        >
                            Nach Status filtern
                        </label>

                        <select
                            id="status-filter"
                            value={filterStatus}
                            onChange={(event) => {
                                setPage(0);
                                setFilterStatus(event.target.value);
                            }}
                            className="rounded border px-4 py-2"
                        >
                            <option value="">Alle Status</option>

                            {statuses.map((item) => (
                                <option key={item} value={item}>
                                    {getStatusLabel(item)}
                                </option>
                            ))}
                        </select>
                    </div>
                </div>
            </section>

            {loading ? (
                <p role="status">Bestellungen werden geladen...</p>
            ) : orders.length === 0 ? (
                <div className="rounded-lg border p-8 text-center">
                    Keine Bestellungen gefunden.
                </div>
            ) : (
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
                                        Benutzer-ID: {order.user_id}
                                    </p>

                                    <p className="text-sm text-gray-600">
                                        Datum:{" "}
                                        {new Date(order.created_at).toLocaleString("de-DE")}
                                    </p>
                                </div>

                                <div className="sm:text-right">
                                    <p className="text-xl font-bold">
                                        {formatPrice(order.total_amount)}
                                    </p>

                                    <p className="mt-1 text-sm text-gray-600">
                                        {items.length} Bestellpositionen
                                    </p>
                                </div>
                            </div>

                            <div className="mt-5 flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-end sm:justify-between">
                                <div>
                                    <label
                                        htmlFor={`status-${order.id}`}
                                        className="mb-1 block text-sm font-medium"
                                    >
                                        Bestellstatus
                                    </label>

                                    <select
                                        id={`status-${order.id}`}
                                        value={order.status}
                                        disabled={updatingId === order.id}
                                        onChange={(event) =>
                                            void handleStatusChange(
                                                order.id,
                                                event.target.value as OrderStatus
                                            )
                                        }
                                        className="rounded border px-3 py-2 disabled:opacity-50"
                                    >
                                        {statuses.map((item) => (
                                            <option key={item} value={item}>
                                                {getStatusLabel(item)}
                                            </option>
                                        ))}
                                    </select>

                                    {updatingId === order.id && (
                                        <p role="status" className="mt-1 text-sm text-gray-500">
                                            Wird aktualisiert...
                                        </p>
                                    )}
                                </div>

                                <Link
                                    href={`/orders/${encodeURIComponent(order.id)}`}
                                    className="rounded border px-4 py-2 text-center hover:bg-gray-50"
                                >
                                    Details ansehen
                                </Link>
                            </div>
                        </article>
                    ))}
                </div>
            )}

            <div className="mt-6 flex items-center justify-between">
                <button
                    type="button"
                    disabled={page === 0 || loading}
                    onClick={() => setPage((current) => current - 1)}
                    className="rounded border px-4 py-2 disabled:opacity-40"
                >
                    Zurück
                </button>

                <p className="text-sm text-gray-600">
                    Seite {totalPages === 0 ? 0 : page + 1} von {totalPages}
                </p>

                <button
                    type="button"
                    disabled={page + 1 >= totalPages || loading}
                    onClick={() => setPage((current) => current + 1)}
                    className="rounded border px-4 py-2 disabled:opacity-40"
                >
                    Weiter
                </button>
            </div>
        </main>
    );
}
