"use client"

import {useState, type FormEvent} from "react";
import Link from "next/link";
import axios from "axios";
import api from "@/lib/api";

interface RegisterRequest{
    email: string;
    password: string;
}

interface RegisterResponse{
    id: string;
    email: string;
    role: string;
}

export default function RegisterPage() {
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [confirmPassword, setConfirmPassword] = useState("");

    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string|null>(null);
    const [success, setSuccess] = useState(false);

    async function handleSubmit(event: FormEvent<HTMLFormElement>){
        event.preventDefault();

        if (loading) return;

        setError(null);
        setSuccess(false);

        const normalizedEmail = email.trim();

        if (password !== confirmPassword) {
            setError("Die Passwörter stimmen nicht überein.");
            return;
        }

        const request: RegisterRequest = {
            email: normalizedEmail,
            password
        };

        setLoading(true);

        try {
            const response = await api.post<RegisterResponse>("/auth/register", request);

            if (response.status === 201) {
                setSuccess(true);
                setPassword("");
                setConfirmPassword("");
            }
        } catch (error: unknown) {
            if (axios.isAxiosError(error)){
                if (error.response?.status === 409){
                    setError("Diese E-Mail-Adresse ist bereits registriert.");
                } else if (error.response?.status === 400) {
                    setError("Bitte überprüfe deine Eingaben.");
                } else {
                    setError("Registrierung fehlgeschlagen. Bitte versuche es erneut.");
                }
            } else {
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
                    Konto erstellen
                </h1>

                <p className="mt-2 text-sm text-gray-600">
                    Registriere dich, um unseren Shop zu nutzen.
                </p>

                {success ? (
                    <div className="mt-8 space-y-4">
                        <div
                            role="status"
                            className="rounded-lg border border-green-200 bg-green-50 p-4 text-sm text-green-800"
                        >
                            Dein Konto wurde erfolgreich erstellt.
                            Du kannst dich jetzt anmelden.
                        </div>

                        <Link
                            href="/login"
                            className="block w-full rounded-lg bg-gray-900 px-4 py-3 text-center font-medium text-white hover:bg-blue-700"
                        >
                            Zum Login
                        </Link>
                    </div>
                ) : (
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
                                autoComplete="new-password"
                                required
                                value={password}
                                onChange={(event) => setPassword(event.target.value)}
                                className="w-full rounded-lg border border-gray-300 px-4 py-3 outline-none text-gray-900 focus:border-blue-600 focus:ring-2 focus:ring-blue-100"
                                placeholder="Passwort eingeben"
                            />
                        </div>

                        <div>
                            <label
                                htmlFor="confirmPassword"
                                className="mb-2 block text-sm font-medium text-gray-700"
                            >
                                Passwort bestätigen
                            </label>

                            <input
                                id="confirmPassword"
                                type="password"
                                autoComplete="new-password"
                                required
                                value={confirmPassword}
                                onChange={(event) => setConfirmPassword(event.target.value)}
                                className="w-full rounded-lg border border-gray-300 px-4 py-3 outline-none text-gray-900 focus:border-blue-600 focus:ring-2 focus:ring-blue-100"
                                placeholder="Passwort wiederholen"
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
                            {loading ? "Registrierung läuft..." : "Registrieren"}
                        </button>
                    </form>
                )}

                {!success && (
                    <p className="mt-6 text-center text-sm text-gray-600">
                        Du hast bereits ein Konto?{" "}
                        <Link
                            href="/login"
                            className="font-medium text-blue-700 hover:underline"
                        >
                            Anmelden
                        </Link>
                    </p>
                )}
            </div>
        </main>
    );
}