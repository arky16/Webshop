import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AuthProvider, useAuth } from "@/context/AuthContext";
import api from "@/lib/api";
import type { AuthUser, LoginResponse } from "@/types/auth";

vi.mock("@/lib/api", () => ({
    default: {
        get: vi.fn(),
    },
}));

function TestComponent() {
    const { user, status, login, logout } = useAuth();

    const testUser: AuthUser = {
        id: "user-123",
        email: "test@example.com",
        role: "USER",
    };

    const loginResponse: LoginResponse = {
        access_token: "test-access-token",
        token_type: "Bearer",
        expires_in: 3600,
        user: testUser,
    };

    return (
        <div>
            <p data-testid="status">{status}</p>
            <p data-testid="user">{user?.email ?? "Kein Benutzer"}</p>

            <button onClick={() => login(loginResponse)}>
                Login
            </button>

            <button onClick={logout}>
                Logout
            </button>
        </div>
    );
}

describe("AuthContext", () => {
    beforeEach(() => {
        localStorage.clear();
        vi.clearAllMocks();
    });

    it("sollte anonymous sein, wenn kein Token vorhanden ist", async () => {
        vi.mocked(api.get).mockResolvedValue({
            data: {},
        } as Awaited<ReturnType<typeof api.get>>);

        render(
            <AuthProvider>
                <TestComponent />
            </AuthProvider>
        );

        await waitFor(() => {
            expect(screen.getByTestId("status")).toHaveTextContent("anonymous");
        });

        expect(screen.getByTestId("user")).toHaveTextContent("Kein Benutzer");
        expect(api.get).not.toHaveBeenCalled();
    });

    it("sollte Benutzer und Token beim Login speichern", async () => {
        const user = userEvent.setup();

        render(
            <AuthProvider>
                <TestComponent />
            </AuthProvider>
        );

        await user.click(screen.getByRole("button", { name: "Login" }));

        expect(screen.getByTestId("status")).toHaveTextContent("authenticated");
        expect(screen.getByTestId("user")).toHaveTextContent("test@example.com");

        expect(localStorage.getItem("token")).toBe("test-access-token");
        expect(localStorage.getItem("user")).toContain("test@example.com");
    });

    it("sollte Benutzer und Token beim Logout entfernen", async () => {
        const user = userEvent.setup();

        localStorage.setItem("token", "test-access-token");
        localStorage.setItem(
            "user",
            JSON.stringify({
                id: "user-123",
                email: "test@example.com",
                role: "USER",
            })
        );

        render(
            <AuthProvider>
                <TestComponent />
            </AuthProvider>
        );

        await user.click(screen.getByRole("button", { name: "Logout" }));

        expect(screen.getByTestId("status")).toHaveTextContent("anonymous");
        expect(screen.getByTestId("user")).toHaveTextContent("Kein Benutzer");

        expect(localStorage.getItem("token")).toBeNull();
        expect(localStorage.getItem("user")).toBeNull();
    });

    it("sollte einen gültigen Token über /account/me überprüfen", async () => {
        localStorage.setItem("token", "valid-token");
        localStorage.setItem(
            "user",
            JSON.stringify({
                id: "user-123",
                email: "test@example.com",
                role: "USER",
            })
        );

        vi.mocked(api.get).mockResolvedValue({
            data: {
                id: "user-123",
                role: "USER",
            },
        } as Awaited<ReturnType<typeof api.get>>);

        render(
            <AuthProvider>
                <TestComponent />
            </AuthProvider>
        );

        await waitFor(() => {
            expect(screen.getByTestId("status")).toHaveTextContent("authenticated");
        });

        expect(api.get).toHaveBeenCalledWith("/account/me");
        expect(screen.getByTestId("user")).toHaveTextContent("test@example.com");
    });

    it("sollte bei HTTP 401 automatisch logout ausführen", async () => {
        localStorage.setItem("token", "invalid-token");
        localStorage.setItem(
            "user",
            JSON.stringify({
                id: "user-123",
                email: "test@example.com",
                role: "USER",
            })
        );

        const unauthorizedError = {
            isAxiosError: true,
            response: {
                status: 401,
            },
        };

        const axios = await import("axios");

        vi.spyOn(axios.default, "isAxiosError").mockReturnValue(true);

        vi.mocked(api.get).mockRejectedValue(unauthorizedError);

        render(
            <AuthProvider>
                <TestComponent />
            </AuthProvider>
        );

        await waitFor(() => {
            expect(screen.getByTestId("status")).toHaveTextContent("anonymous");
        });

        expect(localStorage.getItem("token")).toBeNull();
        expect(localStorage.getItem("user")).toBeNull();
    });
});
