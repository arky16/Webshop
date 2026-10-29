"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import axios from "axios";
import api from "@/lib/api";
import { addToCart } from "@/lib/cart";
import { useAuth } from "@/context/AuthContext";
import type { Product } from "@/types/product";
import { useParams, useRouter } from "next/navigation";

export default function ProductDetailPage() {
    const [product, setProduct] = useState<Product | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const [adding, setAdding] = useState(false);
    const [cartMessage, setCartMessage] = useState("");
    const [cartError, setCartError] = useState("");

    const params = useParams<{ id: string }>();
    const productId = params.id;

    const router = useRouter();
    const { status } = useAuth();

    useEffect(() => {
        let cancelled = false;

        async function loadProduct() {
            setLoading(true);
            setError(null);

            try {
                const response = await api.get<Product>(
                    `/products/${encodeURIComponent(productId)}`
                );

                if (!cancelled) {
                    setProduct(response.data);
                }
            } catch (error: unknown) {
                if (cancelled) return;

                if (axios.isAxiosError(error)) {
                    if (error.response?.status === 404) {
                        setError("Dieses Produkt wurde nicht gefunden.");
                    } else {
                        setError("Produkt konnte nicht geladen werden.");
                    }
                } else {
                    setError("Ein unerwarteter Fehler ist aufgetreten.");
                }
            } finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        }

        void loadProduct();

        return () => {
            cancelled = true;
        };
    }, [productId]);

    function formatPrice(price: string): string {
        const value = Number(price);

        if (!Number.isFinite(value)) {
            return price;
        }

        return new Intl.NumberFormat("de-DE", {
            style: "currency",
            currency: "EUR",
        }).format(value);
    }

    async function handleAddToCart() {
        if (status === "loading" || adding) {
            return;
        }

        setCartMessage("");
        setCartError("");

        if (status === "anonymous") {
            router.push("/login");
            return;
        }

        if (!product || !product.active || product.stock <= 0) {
            setCartError("Dieses Produkt ist aktuell nicht verfügbar.");
            return;
        }

        setAdding(true);

        try {
            await addToCart(product.id, 1);

            setCartMessage(
                "Produkt wurde erfolgreich zum Warenkorb hinzugefügt."
            );
        } catch (error: unknown) {
            if (axios.isAxiosError(error)) {
                const statusCode = error.response?.status;
                const errorCode = error.response?.data?.error;

                if (statusCode === 401) {
                    setCartError(
                        "Deine Anmeldung ist abgelaufen. Bitte melde dich erneut an."
                    );
                } else if (
                    statusCode === 409 ||
                    errorCode === "PRODUCT_UNAVAILABLE"
                ) {
                    setCartError(
                        "Das Produkt ist nicht verfügbar oder der Lagerbestand reicht nicht aus."
                    );
                } else if (statusCode === 400) {
                    setCartError("Die Anfrage ist ungültig.");
                } else {
                    setCartError(
                        "Das Produkt konnte nicht zum Warenkorb hinzugefügt werden."
                    );
                }
            } else {
                setCartError("Ein unerwarteter Fehler ist aufgetreten.");
            }
        } finally {
            setAdding(false);
        }
    }

    return (
        <main className="mx-auto min-h-screen max-w-5xl px-6 py-10">
            <Link
                href="/"
                className="inline-block text-sm font-medium text-blue-700 hover:underline"
            >
                Zurück zum Shop
            </Link>

            {loading ? (
                <p role="status" className="py-16 text-gray-600">
                    Produkt wird geladen...
                </p>
            ) : error ? (
                <div className="mt-8 rounded-xl border border-red-200 bg-red-50 p-6">
                    <p role="alert" className="text-red-700">
                        {error}
                    </p>

                    <Link
                        href="/"
                        className="mt-4 inline-block font-medium text-blue-700 hover:underline"
                    >
                        Zur Produktübersicht
                    </Link>
                </div>
            ) : product ? (
                <article className="mt-8 rounded-2xl border border-gray-200 bg-white p-6 shadow-sm sm:p-10">
                    <div className="mb-6 flex flex-wrap items-center gap-3">
            <span className="rounded-full bg-blue-50 px-3 py-1 text-sm font-medium text-blue-700">
              {product.category ?? "Allgemein"}
            </span>

                        <span className="text-sm text-gray-500">
              {product.stock > 0 && product.active
                  ? "Auf Lager"
                  : "Nicht auf Lager"}
            </span>
                    </div>

                    <h1 className="text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
                        {product.name}
                    </h1>

                    <p className="mt-6 whitespace-pre-line leading-7 text-gray-600">
                        {product.description}
                    </p>

                    <div className="mt-8 border-t border-gray-200 pt-6">
                        <p className="text-sm text-gray-500">Preis</p>

                        <p className="mt-1 text-3xl font-bold text-gray-900">
                            {formatPrice(product.price)}
                        </p>

                        <p className="mt-3 text-sm text-gray-500">
                            Lagerbestand: {product.stock}
                        </p>
                    </div>

                    <div className="mt-8">
                        <button
                            type="button"
                            disabled={
                                product.stock <= 0 ||
                                !product.active ||
                                adding ||
                                status === "loading"
                            }
                            onClick={() => void handleAddToCart()}
                            className="w-full rounded-lg bg-gray-900 px-6 py-3 font-medium text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-gray-300 sm:w-auto"
                        >
                            {adding
                                ? "Wird hinzugefügt..."
                                : status === "loading"
                                    ? "Bitte warten..."
                                    : "Zum Warenkorb"}
                        </button>
                    </div>

                    {cartMessage && (
                        <div className="mt-5 rounded-lg border border-green-200 bg-green-50 p-4">
                            <p role="status" className="text-sm text-green-800">
                                {cartMessage}
                            </p>

                            <Link
                                href="/cart"
                                className="mt-2 inline-block font-medium text-green-800 underline"
                            >
                                Zum Warenkorb
                            </Link>
                        </div>
                    )}

                    {cartError && (
                        <div className="mt-5 rounded-lg border border-red-200 bg-red-50 p-4">
                            <p role="alert" className="text-sm text-red-700">
                                {cartError}
                            </p>

                            {cartError.includes("erneut an") && (
                                <Link
                                    href="/login"
                                    className="mt-2 inline-block font-medium text-blue-700 underline"
                                >
                                    Zum Login
                                </Link>
                            )}
                        </div>
                    )}
                </article>
            ) : null}
        </main>
    );
}
