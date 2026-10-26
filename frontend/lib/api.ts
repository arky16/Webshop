import axios from "axios";

const api = axios.create({
    baseURL:
        process.env.NEXT_PUBLIC_API_URL ??
        "http://localhost:8081/api"
});

export default api;