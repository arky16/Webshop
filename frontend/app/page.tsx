"use client";

import {useEffect, useState} from "react";
import api from "@/lib/api";
import type {Product, ProductListResponse} from "@/types/product";
import axios from "axios";
import Link from "next/link";
import {useRouter} from "next/navigation";
import {addToCart} from "@/lib/cart";
import {useAuth} from "@/context/AuthContext";

export default function Home() {
    const [products, setProducts] = useState<Product[]>([]);
    const [page, setPage] = useState(0);
    const [totalPages, setTotalPages] = useState(0);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [retry, setRetry] = useState(0);
    const [searchInput, setSearchInput] = useState("");
    const [search, setSearch] = useState("");
    const [categoryInput, setCategoryInput] = useState("");
    const [minPriceInput, setMinPriceInput] = useState("");
    const [maxPriceInput, setMaxPriceInput] = useState("");
    const [category, setCategory] = useState("");
    const [minPrice, setMinPrice] = useState("");
    const [maxPrice, setMaxPrice] = useState("");
    const [sortByInput, setSortByInput] = useState("created_at");
    const [orderInput, setOrderInput] = useState("desc");
    const [sortBy, setSortBy] = useState("created_at");
    const [order, setOrder] = useState("desc");
    const router = useRouter();
    const {status} = useAuth();
    const [addingProductId, setAddingProductId] = useState<string|null>(null);
    const [cartMessage, setCartMessage] = useState("");
    const [cartError, setCartError] = useState("");

    const pageSize = 10;

    useEffect(() => {
        let cancelled = false;

        async function loadProducts(){
            setLoading(true);
            setError(null);

            try {
                const response = await api.get<ProductListResponse>("/products", {
                    params: {
                        page,
                        size: pageSize,
                        ...(search.trim() ? {search: search.trim()}: {}),
                        ...(category ? {category}: {}),
                        ...(minPrice !== "" ? {min_price: minPrice}: {}),
                        ...(maxPrice !== "" ? {max_price: maxPrice}: {}),
                        sortBy: sortBy,
                        order,
                    }
                });

                if (cancelled) return;

                setProducts(response.data.products);
                setTotalPages(response.data.pagination.total_pages);
            } catch (error: unknown) {
                if (cancelled) return;

                console.error("=== PRODUCT REQUEST DEBUG ===");
                console.error("Error value:", error);
                console.error("Error type:", typeof error);
                console.error("Is Axios Error:", axios.isAxiosError(error));
                console.error("API Base URL:", api.defaults.baseURL);

                if (error instanceof Error) {
                    console.error("Error name:", error.name);
                    console.error("Error message:", error.message);
                    console.error("Error stack:", error.stack);
                }

                if (axios.isAxiosError(error)) {
                    console.error("Axios details:", {
                        message: error.message,
                        code: error.code,
                        status: error.response?.status,
                        response: error.response?.data,
                        url: error.config?.url,
                    });
                }

                setError("Produkte konnten nicht geladen werden");
            } finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        }

        void loadProducts();

        return () => {
            cancelled = true;
        }
    }, [page, retry, search, category, minPrice, maxPrice, sortBy, order]);

    function formatPrice(price: string): string {
        const value = Number(price);

        if (!Number.isFinite(value)) {
            return price;
        }

        return new Intl.NumberFormat("de-DE", {
            style: "currency",
            currency: "EUR"
        }).format(value);
    }

    async function handleAddToCart(productId: string) {
        if (status === "loading" || addingProductId !== null) {
            return;
        }

        setCartMessage("");
        setCartError("");

        if (status === "anonymous") {
            router.push("/login");
            return;
        }

        setAddingProductId(productId);

        try {
            await addToCart(productId, 1);

            setCartMessage("Produkt wurde zum Warenkorb hinzugefügt.");
        } catch (error: unknown) {
            if (axios.isAxiosError(error)) {
                const statusCode = error.response?.status;
                const errorCode = error.response?.data?.error;

                if (statusCode === 401) {
                    setCartError("Bitte melde dich erneut an.");
                } else if (
                    statusCode === 409 ||
                    errorCode === "PRODUCT_UNAVAILABLE"
                ) {
                    setCartError(
                        "Produkt nicht verfügbar oder Lagerbestand unzureichend."
                    );
                } else if (statusCode === 400) {
                    setCartError("Die Anfrage ist ungültig.");
                } else {
                    setCartError("Produkt konnte nicht hinzugefügt werden.");
                }
            } else {
                setCartError("Ein unerwarteter Fehler ist aufgetreten.");
            }
        } finally {
            setAddingProductId(null);
        }
    }


    return (
        <main className="mx-auto min-h-screen max-w-7xl px-6 py-10">
            <header className="mb-10">
                <h1 className="text-4xl font-bold tracking-tight">
                    Unsere Produkte
                </h1>

                <p className="mt-3 text-gray-600">
                    Entdecke unsere aktuellen Produkte
                </p>
            </header>

            <form
                className="mx-auto mb-8 flex w-full max-w-2xl gap-3"
                onSubmit={(event) => {
                    event.preventDefault();
                    setPage(0);
                    setSearch(searchInput.trim());
                }}
            >
                <div className="flex-1">
                    <label htmlFor="product-search" className="sr-only">
                        Produktsuche
                    </label>

                    <input
                        id="product-search"
                        type="search"
                        value={searchInput}
                        onChange={(event) => setSearchInput(event.target.value)}
                        placeholder="Produkte suchen..."
                        className="w-full rounded-lg border border-gray-300 px-4 py-3 outline-none focus:border-blue-600"
                    />
                </div>

                <button
                    type="submit"
                    className="rounded-lg bg-gray-900 px-6 py-3 font-medium text-white hover:bg-blue-700"
                >
                    Suchen
                </button>

                <button
                    type="button"
                    onClick={() => {
                        setSearchInput("");
                        setSearch("");
                        setPage(0);
                    }}
                    className="rounded-lg bg-gray-900 px-6 py-3 font-medium text-white hover:bg-blue-700"
                >
                    Zurücksetzen
                </button>
            </form>


            <aside className="mb-8 space-y-4 border-b border-gray-200 bg-white p-5 lg:absolute lg:left-100 lg:top-65 lg:mb-0 lg:w-64 lg:border-b-0 lg:border-r">
                <form
                    className="space-y-4 rounded-2xl border border-gray-200 bg-white p-5 lg:rounded-none lg:border-0 lg:p-0"
                    onSubmit={(event) => {
                        event.preventDefault();
                        setPage(0);
                        setCategory(categoryInput.trim());
                    }}
                >
                    <h2 className="text-lg font-semibold text-gray-900">Kategorie</h2>

                    <div>
                        <label htmlFor="category" className="sr-only">
                            Kategorie
                        </label>

                        <input
                            id="category"
                            type="text"
                            value={categoryInput}
                            onChange={(event) => setCategoryInput(event.target.value)}
                            placeholder="z. B. Computerzubehör"
                            className="w-full rounded-lg border border-gray-300 text-gray-900 px-4 py-3 outline-none focus:border-blue-600"
                        />
                    </div>

                    <div className="flex flex-col gap-3">
                        <button
                            type="submit"
                            className="rounded-lg bg-gray-900 px-6 py-3 font-medium text-white hover:bg-blue-700"
                        >
                            Kategorie anwenden
                        </button>

                        <button
                            type="button"
                            onClick={() => {
                                setCategoryInput("");
                                setCategory("");
                                setPage(0);
                            }}
                            className="rounded-lg border border-gray-300 px-6 py-3 font-medium text-gray-900 hover:bg-gray-200"
                        >
                            Zurücksetzen
                        </button>
                    </div>
                </form>

                <form
                    className="space-y-4 rounded-2xl border border-gray-200 bg-white p-5 lg:rounded-none lg:border-0 lg:border-t lg:p-0 lg:pt-4"
                    onSubmit={(event) => {
                        event.preventDefault();
                        setPage(0);
                        setMinPrice(minPriceInput.trim());
                        setMaxPrice(maxPriceInput.trim());
                        setSortBy(sortByInput);
                        setOrder(orderInput);
                    }}
                >
                    <h2 className="text-lg font-semibold text-gray-900">Preis</h2>

                    <div>
                        <label htmlFor="min-price" className="mb-2 block text-sm font-medium text-gray-700">
                            Mindestpreis (€)
                        </label>

                        <input
                            id="min-price"
                            type="number"
                            min="0"
                            step="0.01"
                            value={minPriceInput}
                            onChange={(event) => setMinPriceInput(event.target.value)}
                            placeholder="0,00"
                            className="w-full rounded-lg border border-gray-300 px-4 py-3 outline-none text-gray-900 focus:border-blue-600"
                        />
                    </div>

                    <div>
                        <label htmlFor="max-price" className="mb-2 block text-sm font-medium text-gray-700">
                            Maximalpreis (€)
                        </label>

                        <input
                            id="max-price"
                            type="number"
                            min="0"
                            step="0.01"
                            value={maxPriceInput}
                            onChange={(event) => setMaxPriceInput(event.target.value)}
                            placeholder="100,00"
                            className="w-full rounded-lg border border-gray-300 px-4 py-3 outline-none text-gray-900 focus:border-blue-600"
                        />
                    </div>

                    <div className="flex flex-col gap-3">
                        <button
                            type="submit"
                            className="rounded-lg bg-gray-900 px-6 py-3 font-medium text-white hover:bg-blue-700"
                        >
                            Preis anwenden
                        </button>

                        <button
                            type="button"
                            onClick={() => {
                                setMinPriceInput("");
                                setMaxPriceInput("");
                                setMinPrice("");
                                setMaxPrice("");
                                setPage(0);
                            }}
                            className="rounded-lg border border-gray-300 px-6 py-3 font-medium text-gray-900 hover:bg-gray-200"
                        >
                            Zurücksetzen
                        </button>
                    </div>

                    <div className="flex flex-col gap-3">
                        <div>
                            <label
                                htmlFor="sort-by"
                                className="mb-2 block text-sm font-medium text-gray-700"
                            >
                                Sortieren nach
                            </label>

                            <select
                                id="sort-by"
                                value={sortByInput}
                                onChange={(event) => setSortByInput(event.target.value)}
                                className="w-full rounded-lg border border-gray-300 bg-white px-4 py-3 text-gray-900"
                            >
                                <option value="created_at">Neueste Produkte</option>
                                <option value="name">Name</option>
                                <option value="price">Preis</option>
                            </select>
                        </div>

                        <div>
                            <label
                                htmlFor="sort-order"
                                className="mb-2 block text-sm font-medium text-gray-700"
                            >
                                Reihenfolge
                            </label>

                            <select
                                id="sort-order"
                                value={orderInput}
                                onChange={(event) => setOrderInput(event.target.value)}
                                className="w-full rounded-lg border border-gray-300 bg-white px-4 py-3 text-gray-900"
                            >
                                <option value="asc">Aufsteigend</option>
                                <option value="desc">Absteigend</option>
                            </select>
                        </div>
                    </div>
                </form>
            </aside>

            {loading ? (
                <p role="status" className="py-10 text-gray-600">
                    Produkte werden geladen...
                </p>
            ): error ? (
                <div className="rounded-xl border border-red-200 bg-red-50 p-6">
                    <p role="alert" className="text-red-700">
                        {error}
                    </p>

                    <button
                        type="button"
                        onClick={() => setRetry((value) => value + 1)}
                        className="mt-4 rounded-lg bg-red-600 px-4 py-2 text-white hover:bg-red-700"
                    >
                        Erneut versuchen
                    </button>
                </div>
            ) : products.length === 0 ? (
               <p className="py-10 text-gray-600">
                   Keine Produkte gefunde.
               </p>
            ) : (
                <>
                    <section
                        aria-label="Produktliste"
                        className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3"
                    >
                        {products.map((product) => (
                            <article
                                key={product.id}
                                className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm transition hover:shadow-md"
                            >
                                <div className="mb-4 flex items-start justify-between gap-3">
                                    <span className="rounded-full bg-blue-50 px-3 py-1 text-xs font-medium text-blue-700">
                                        {product.category ?? "Allgemein"}
                                    </span>

                                    <span className="text-ts text-gray-500">
                                        {product.stock > 0 ? "Auf Lager" : "Nicht auf Lager"}
                                    </span>
                                </div>

                                <h2 className="text-xl font-semibold text-gray-900">
                                    <Link
                                        href={`/products/${product.id}`}
                                        className="transition hover:text-blue-700"
                                    >
                                        {product.name}
                                    </Link>
                                </h2>

                                <p className="mt-3 min-h-12 text-sm leading-6 text-gray-600">
                                    {product.description}
                                </p>

                                <div className="mt-6 flex items-end justify-between gap-3">
                                    <div>
                                        <p className="text-2xl font-bold text-gray-900">
                                            {formatPrice(product.price)}
                                        </p>

                                        <p className="mt-1 text-xs text-gray-500">
                                            Bestand: {product.stock}
                                        </p>
                                    </div>

                                    <button
                                        type="button"
                                        disabled={
                                            product.stock <= 0 ||
                                            !product.active ||
                                            addingProductId !== null ||
                                            status === "loading"
                                        }
                                        onClick={() => void handleAddToCart(product.id)}
                                        className="rounded-lg bg-gray-900 px-4 py-2 text-sm font-medium text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-gray-300"
                                    >
                                        {addingProductId === product.id
                                            ? "Wird hinzugefügt..."
                                            : "Zum Warenkorb"}
                                    </button>
                                </div>
                                {cartMessage && (
                                    <div className="my-4 rounded-lg border border-green-200 bg-green-50 p-4">
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
                                    <p role="alert" className="my-4 text-sm text-red-700">
                                        {cartError}
                                        {cartError.includes("erneut anmelden") && (
                                            <>
                                                {" "}
                                                <Link href="/login" className="underline">
                                                    Zum Login
                                                </Link>
                                            </>
                                        )}
                                    </p>
                                )}
                            </article>
                        ))}
                    </section>

                    <nav
                        aria-label="Produktseiten"
                        className="mt-10 flex items-center justify-center gap-6"
                    >
                        <button
                            type="button"
                            disabled={page <= 0}
                            onClick={() => setPage((value) => value - 1)}
                            className="rounded-lg border px-4 py-2 disabled:cursor-not-allowed disabled:opacity-40"
                        >
                            Zurück
                        </button>

                        <span className="text-sm text-gray-600">
                            Seite {page + 1} von {Math.max(totalPages, 1)}
                        </span>

                        <button
                            type="button"
                            disabled={page + 1 >= totalPages}
                            onClick={() => setPage((value) => value + 1)}
                            className="rounded-lg border px-4 py-2 disabled:cursor-not-allowed disabled:opacity-40"
                        >
                            Weiter
                        </button>
                    </nav>
                </>
            )}
        </main>
    );
}