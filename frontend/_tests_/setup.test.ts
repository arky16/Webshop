import { describe, it, expect } from "vitest";

describe("Frontend Test Setup", () => {
    it("sollte einfache Berechnungen testen können", () => {
        expect(2 + 2).toBe(4);
    });

    it("sollte Text vergleichen können", () => {
        expect("Webshop").toBe("Webshop");
    });
});
