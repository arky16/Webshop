import { describe, it, expect, beforeEach } from "vitest";
import api from "@/lib/api";

describe("API Client", () => {
    beforeEach(() => {
        localStorage.clear();
    });

    it("sollte die richtige Backend-URL verwenden", () => {
        expect(api.defaults.baseURL).toBe(
            process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8081/api"
        );
    });

    it("sollte den JWT-Token im Authorization-Header setzen", async () => {
        localStorage.setItem("token", "test-access-token");

        let authorizationHeader: string | undefined;

        await api.get("/test", {
            adapter: async (config) => {
                authorizationHeader = config.headers.get("Authorization")?.toString();

                return {
                    data: {},
                    status: 200,
                    statusText: "OK",
                    headers: {},
                    config,
                };
            },
        });

        expect(authorizationHeader).toBe("Bearer test-access-token");
    });

    it("sollte keinen Authorization-Header setzen, wenn kein Token vorhanden ist", async () => {
        let authorizationHeader: string | undefined;

        await api.get("/test", {
            adapter: async (config) => {
                authorizationHeader = config.headers.get("Authorization")?.toString();

                return {
                    data: {},
                    status: 200,
                    statusText: "OK",
                    headers: {},
                    config,
                };
            },
        });

        expect(authorizationHeader).toBeUndefined();
    });
});

