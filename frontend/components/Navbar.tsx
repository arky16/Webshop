"use client"

import Link from "next/link";
import {useAuth} from "@/context/AuthContext";

export default function Navbar() {
    const {user, status, logout} = useAuth();

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
                                className="text-sm font-medium text-gray-700 hover:text-blue-700"
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
                </div>
            </nav>
        </header>
    );
}