"use client";

import {useEffect, useState} from "react";
import Link from "next/link";
import axios from "axios";
import api from "@/lib/api";
import type {Product} from "@/types/product";
import {useParams} from "next/navigation";

export default function ProductDetailPage(){
    const [product, setProduct] = useState<Product | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const params = useParams<{id: string}>();
    const productId = params.id;

    useEffect(() => {
        let cancelled = false;

        async function loadProduct(){
            setLoading(true);
            setError(null);

            try {
                const response = await api.get<Product>(`/products/${encodeURIComponent(productId)}`);

                if (!cancelled) {
                    setProduct(response.data);
                }
            }catch (error: unknown) {
                if (cancelled) return;

                if (axios.isAxiosError(error)) {
                    if (error.response?.status === 404){
                        setError("Dieses Produkt wurde nicht gefunden.");
                    }else {
                        setError("Produkt konnte nicht geladen werden.");
                    }
                }else {
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
        }
    }, [productId]);

    function formatPrice(price: string): string {
        const value = Number(price);

        if(!Number.isFinite(value)) {
            return price;
        }

        return new Intl.NumberFormat("de-DE", {
            style: "currency",
            currency: "EUR"
        }).format(value);
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
                            {product.stock > 0 ? "Auf Lager" : "Nicht auf Lager"}
                        </span>
                    </div>

                    <h1 className="text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">
                        {product.name}
                    </h1>

                    <p className="mt-6 whitespace-pre-line leading-7 text-gray-600">
                        {product.description}
                    </p>

                    <div className="mt-8 border-t border-gray-200 pt-6">
                        <p className="text-sm text-gray-500">
                            Preis
                        </p>

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
                            disabled={product.stock <= 0 || !product.active}
                            className="w-full rounded-lg bg-gray-900 px-6 py-3 font-medium text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-gray-300 sm:w-auto"
                            onClick={() => {
                                alert("Den Warenkorb ist noch nicht fertig.");
                            }}
                        >
                            Zum Warenkorb
                        </button>
                    </div>
                </article>
            ) : null}
        </main>
    );
}