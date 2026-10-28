import axios from "axios";

const api = axios.create({
    baseURL:
        process.env.NEXT_PUBLIC_API_URL ??
        "http://localhost:8081/api"
});

api.interceptors.request.use((config) => {
    if (typeof window !== "undefined"){
        const token = localStorage.getItem("token");

        if (token) {
            config.headers.set("Authorization", `Bearer ${token}`);
        }
    }

    return config;
})

export default api;