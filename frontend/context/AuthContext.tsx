"use client"

import {createContext, useCallback, useContext, useState, useEffect, ReactNode} from "react";
import axios from "axios";
import api from "@/lib/api";
import type {AuthUser, LoginResponse} from "@/types/auth";

type AuthStatus = "loading" | "authenticated" | "anonymous";

interface AuthContextType {
    user: AuthUser | null;
    status: AuthStatus;
    login: (data: LoginResponse) => void;
    logout: () => void;
}

const AuthContext = createContext<AuthContextType | null>(null);

interface AuthProviderProps{
    children: ReactNode;
}

interface MeResponse {
    id: string;
    role: string;
}

export function AuthProvider({children}: AuthProviderProps) {
    const [user, setUser] = useState<AuthUser | null>(null);
    const [status, setStatus] = useState<AuthStatus>("loading");

    const logout = useCallback(() => {
        localStorage.removeItem("token");
        localStorage.removeItem("user");
        setUser(null);
        setStatus("anonymous");
    }, []);

    const login = useCallback((data: LoginResponse) => {
        localStorage.setItem("token", data.access_token);
        localStorage.setItem("user", JSON.stringify(data.user));

        setUser(data.user);
        setStatus("authenticated");
    }, []);

    useEffect(() => {
        let cancelled = false;

        async function checkAuth() {
            const token = localStorage.getItem("token");

            if (!token) {
                if (!cancelled) {
                    setStatus("anonymous");
                }
                return;
            }

            try {
                const response = await api.get<MeResponse>("/account/me");

                if (cancelled) return;

                const cachedUser = localStorage.getItem("user");

                let email = "";

                if (cachedUser) {
                    try {
                        const parsed: unknown = JSON.parse(cachedUser);

                        if (
                            typeof parsed === "object" &&
                            parsed !== null &&
                            "email" in parsed &&
                            typeof parsed.email === "string"
                        ) {
                            email = parsed.email;
                        }
                    } catch {}
                }

                const verifiedUser: AuthUser = {
                    id: response.data.id,
                    role: response.data.role,
                    email
                };

                localStorage.setItem("user", JSON.stringify(verifiedUser));

                setUser(verifiedUser);
                setStatus("authenticated");
            } catch (error: unknown) {
                if (cancelled) return;

                if (axios.isAxiosError(error) && error.response?.status === 401) {
                    logout();
                    return;
                }

                setStatus("loading");
            }
        }

        void checkAuth();

        return () => {
            cancelled = true;
        };
    }, [logout]);

    return (
        <AuthContext.Provider value={{user, status, login, logout}}>
            {children}
        </AuthContext.Provider>
    );
}

export function useAuth() {
    const context = useContext(AuthContext);

    if (!context) {
        throw new Error("useAuth muss innerhalb von AuthProvider verwendet werden.")
    }

    return context;
}