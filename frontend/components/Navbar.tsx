"use client"

import Link from "next/link";
import {useAuth} from "@/context/AuthContext";
import {useCartCount} from "@/context/CartContext";

export default function Navbar() {
    const {user, status, logout} = useAuth();
    const {itemCount} = useCartCount();

    return(
        <header className="border-b border-gray-200 bg-white">
            <nav className="mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-4 px-6 py-4">
                <Link
                    href="/"
                    className="text-xl font-bold tracking-tight text-gray-900"
                >
                    Webshop
                </Link>

                <div className="flex flex-wrap items-center gap-4">
                    {status === "loading" ? (
                        <span
                            role="status"
                            className="text-sm text-gray-500"
                        >
                            Authentifizierung wird geprüft...
                        </span>
                    ) : status === "authenticated" ? (
                        <>
                            <span className="hidden text-sm text-gray-600 sm:inline">
                                {user?.email || "Angemeldet"}
                            </span>

                            <button
                                type="button"
                                onClick={logout}
                                className="rounded-lg border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 transition hover:bg-gray-100"
                            >
                                Logout
                            </button>
                        </>
                    ) : (
                        <>
                            <Link
                                href="/login"
                                className="rounded-lg border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 transition hover:bg-gray-100"
                            >
                                Login
                            </Link>

                            <Link
                                href="/register"
                                className="rounded-lg bg-gray-900 px-4 py-2 text-sm font-medium text-white transition hover:bg-blue-700"
                            >
                                Registrieren
                            </Link>
                        </>
                    )}

                    <Link
                        href="/cart"
                        aria-label={`Warenkorb, ${itemCount} Artikel`}
                        className="rounded-lg bg-blue-700 px-4 py-2 text-sm font-medium text-white transition hover:bg-black"
                    >
                        <span>Warenkorb</span>

                        <span
                            aria-hidden="true"
                            className="inline-flex min-w-6 h-6 items-center justify-center rounded-full px-1.5 text-xs font-bold text-white"
                        >
                            {itemCount}
                        </span>
                    </Link>

                    {status === "authenticated" && (
                        <Link
                            href="/orders"
                            className="rounded-lg bg-blue-700 px-4.5 py-2.5 text-sm font-medium text-white transition hover:bg-black"
                        >
                            Bestellungen
                        </Link>
                    )}

                    {status === "authenticated" && user?.role === "ADMIN" && (
                        <Link
                            href="/admin/orders"
                            className="rounded-lg bg-blue-700 px-4.5 py-2.5 text-sm font-medium text-white transition hover:bg-black"
                        >
                            Admin-Bestellungen
                        </Link>
                    )}
                </div>
            </nav>
        </header>
    );
}