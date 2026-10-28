"use client"

import {useState, type FormEvent} from "react";
import {useRouter} from "next/navigation";
import Link from "next/link";
import axios from "axios";
import api from "@/lib/api";
import type {LoginRequest, LoginResponse} from "@/types/auth";
import {useAuth} from "@/context/AuthContext";

export default function LoginPage() {
    const router = useRouter();

    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string|null>(null);
    const {login} = useAuth();

    async function handleSubmit(event: FormEvent<HTMLFormElement>){
        event.preventDefault();

        if (loading) return;

        setLoading(true);
        setError(null);

        const credentials: LoginRequest = {
            email: email.trim(),
            password
        };

        try {
            const response = await api.post<LoginResponse>(
                "/auth/login",
                credentials
            );

            const data = response.data;

            if (!data.access_token || data.token_type.toLowerCase() !== "bearer"){
                setError("Ungültige Antwort vom Server");
                return;
            }

            login(data);

            router.replace("/");
        } catch (error: unknown) {
            if (axios.isAxiosError(error)) {
                if (error.response?.status === 401) {
                    setError("E-Mail oder Password ist falsch.");
                } else if (error.response?.status === 400) {
                    setError("Bitte überprüfe deine Eingaben.");
                } else {
                    setError("Login fehlgeschlagen. Bitte versuche es erneut.");
                }
            }else {
                setError("Ein unerwarteter Fehler ist aufgetreten.")
            }
        } finally {
            setLoading(false);
        }
    }

    return (
        <main className="flex min-h-screen items-center justify-center bg-gray-50 px-4 py-10">
            <div className="w-full max-w-md rounded-2xl border border-gray-200 bg-white p-6 shadow-sm sm:p-8">
                <Link
                    href="/"
                    className="text-sm font-medium text-blue-700 hover:underline"
                >
                    Zurück zum Shop
                </Link>

                <h1 className="mt-6 text-3xl font-bold text-gray-900">
                    Anmelden
                </h1>

                <p className="mt-2 text-sm text-gray-600">
                    Melde dich an, um deinen Warenkorb zu verwenden.
                </p>

                <form onSubmit={handleSubmit} className="mt-8 space-y-5">
                    <div>
                        <label
                            htmlFor="email"
                            className="mb-2 block text-sm font-medium text-gray-700"
                        >
                            E-Mail-Adresse
                        </label>

                        <input
                            id="email"
                            type="email"
                            autoComplete="email"
                            required
                            maxLength={254}
                            value={email}
                            onChange={(event) => setEmail(event.target.value)}
                            className="w-full rounded-lg border border-gray-300 px-4 py-3 outline-none text-gray-900 focus:border-blue-600 focus:ring-2 focus:ring-blue-100"
                            placeholder="name@example.com"
                        />
                    </div>

                    <div>
                        <label
                            htmlFor="password"
                            className="mb-2 block text-sm font-medium text-gray-700"
                        >
                            Passwort
                        </label>

                        <input
                            id="password"
                            type="password"
                            autoComplete="current-password"
                            required
                            value={password}
                            onChange={(event) => setPassword(event.target.value)}
                            className="w-full rounded-lg border border-gray-300 px-4 py-3 outline-none text-gray-900 focus:border-blue-600 focus:ring-2 focus:ring-blue-100"
                            placeholder="Dein Passwort"
                        />
                    </div>

                    {error && (
                        <p
                            role="alert"
                            className="rounded-lg bg-red-50 p-3 text-sm text-red-700"
                        >
                            {error}
                        </p>
                    )}

                    <button
                        type="submit"
                        disabled={loading}
                        className="w-full rounded-lg bg-gray-900 px-4 py-3 font-medium text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-60"
                    >
                        {loading ? "Anmeldung läuft..." : "Anmelden"}
                    </button>
                </form>

                <p className="mt-6 text-center text-sm text-gray-600">
                    Noch kein Konto?{" "}
                    <Link
                        href="/register"
                        className="font-medium text-blue-700 hover:underline"
                    >
                        Registrieren
                    </Link>
                </p>
            </div>
        </main>
    );
}