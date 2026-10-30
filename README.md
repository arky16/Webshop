# Fullstack Webshop

A fullstack webshop application built with **Rust, Actix-web, PostgreSQL, Redis and Next.js**.

The application provides product management, user authentication, shopping cart functionality and order processing. The entire application can be started using Docker Compose.

## Features

- User registration and login
- JWT-based authentication and authorization
- Role-based access control (USER / ADMIN)
- Product listing with pagination and filtering
- Product management (create, update and delete)
- Shopping cart management
- Checkout and order processing
- Order history and order details
- Admin order management and status updates
- Redis caching for product data
- Redis-based rate limiting for authentication
- PostgreSQL database with SQLx migrations
- Dockerized application environment

## Technologies

| Component | Technologies |
|---|---|
| Backend | Rust, Actix-web |
| Database | PostgreSQL 17 |
| Database access | SQLx |
| Cache / Rate Limiting | Redis 7 |
| Authentication | JWT, Argon2 |
| Frontend | Next.js 16, React 19, TypeScript |
| Styling | Tailwind CSS |
| Frontend HTTP Client | Axios |
| Frontend Testing | Vitest, React Testing Library |
| Containerization | Docker, Docker Compose |

## Architecture

The application consists of four Docker services:

```text
                 Browser
                    |
                    v
             Next.js Frontend
              localhost:3000
                    |
                    v
              Actix-web API
              localhost:8081
                    |
              +-----+-----+
              |           |
              v           v
          PostgreSQL     Redis
           db:5432      redis:6379
```

- **Frontend:** User interface and communication with the backend API.
- **Backend:** REST API, authentication, business logic and order processing.
- **PostgreSQL:** Persistent storage for users, products, carts and orders.
- **Redis:** Product caching and rate limiting.

## Requirements

Make sure the following tools are installed:

- [Git](https://git-scm.com/)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/)
- Docker Compose (included with Docker Desktop)

## Installation

### 1. Clone the repository

```bash
git clone YOUR_GITHUB_REPOSITORY_URL
cd Webshop
```

### 2. Configure environment variables

Create a `.env` file in the project root:

```env
DB_PASSWORD=your_secure_database_password
```

Create `backend/.env`:

```env
JWT_SECRET=your_long_random_secret
```

The backend environment file is loaded by Docker Compose. The database connection URL and Redis URL are configured by Docker Compose.

For local frontend development, create `frontend/.env.local`:

```env
NEXT_PUBLIC_API_URL=http://localhost:8081/api
```

**Important:** Never commit real passwords, JWT secrets or other sensitive environment variables to GitHub.

### 3. Start the application

From the project root, execute:

```bash
docker compose up -d --build
```

Docker Compose builds and starts all four services:

- PostgreSQL
- Redis
- Backend
- Frontend

Database migrations are executed automatically when the backend starts.

### 4. Check running containers

```bash
docker compose ps
```

### 5. Access the application

| Service | URL |
|---|---|
| Frontend | http://localhost:3000 |
| Backend API | http://localhost:8081 |
| Backend Healthcheck | http://localhost:8081/health |

The health endpoint checks the backend, PostgreSQL and Redis connections.

## API Endpoints

All API endpoints use the base URL:

```text
http://localhost:8081/api
```

### Authentication

| Method | Endpoint | Description | Access |
|---|---|---|---|
| POST | `/auth/register` | Register a new user | Public |
| POST | `/auth/login` | Login and receive JWT | Public |
| GET | `/account/me` | Get current user information | Authenticated |

### Products

| Method | Endpoint | Description | Access |
|---|---|---|---|
| GET | `/products` | List products with pagination and filtering | Public |
| GET | `/products/{id}` | Get product details | Public |
| POST | `/products` | Create a product | ADMIN |
| PUT | `/products/{id}` | Update a product | ADMIN |
| DELETE | `/products/{id}` | Delete a product | ADMIN |

### Shopping Cart

| Method | Endpoint | Description | Access |
|---|---|---|---|
| GET | `/cart` | Get shopping cart | Authenticated |
| POST | `/cart` | Add product to cart | Authenticated |
| DELETE | `/cart/{product_id}` | Remove product from cart | Authenticated |

### Orders

| Method | Endpoint | Description | Access |
|---|---|---|---|
| POST | `/orders/checkout` | Create an order | Authenticated |
| GET | `/orders` | Get current user's orders | Authenticated |
| GET | `/orders/{order_id}` | Get order details | Authenticated |

### Admin Orders

| Method | Endpoint | Description | Access |
|---|---|---|---|
| GET | `/admin/orders` | Get all orders | ADMIN |
| PATCH | `/admin/orders/{order_id}/status` | Update order status | ADMIN |

Authenticated endpoints require a JWT access token:

```http
Authorization: Bearer YOUR_ACCESS_TOKEN
```

## Order Processing

The checkout process follows these steps:

1. The user adds products to the shopping cart.
2. The backend validates product availability and stock.
3. A database transaction creates the order and its order items.
4. Product stock is reduced.
5. The shopping cart is cleared.
6. The order is created with the initial status `PENDING`.

Order processing uses database transactions to maintain data consistency.

## Caching and Rate Limiting

Redis is used for two main purposes:

**Caching**
- Product lists and individual product details are cached.
- Cached entries have a TTL of 60 seconds.
- Product cache entries are invalidated after create, update and delete operations.
- If Redis is unavailable, the backend falls back to PostgreSQL for product requests.

**Rate Limiting**
- Login attempts are limited to five failed attempts within 60 seconds per IP address.
- Successful login resets the failed-attempt counter.
- Registration requests also have rate limiting.

## Database Migrations

SQLx migrations are located in:

```text
backend/migrations/
```

The migrations are executed automatically when the backend starts.

## Testing

### Backend

Navigate to the backend directory:

```bash
cd backend
cargo test
```

### Frontend

Navigate to the frontend directory:

```bash
cd frontend
```

Run frontend tests:

```bash
npm run test:run
```

Run ESLint:

```bash
npm run lint
```

Check TypeScript:

```bash
npx tsc --noEmit
```

Build the frontend:

```bash
npm run build
```

## Useful Docker Commands

Start all services:

```bash
docker compose up -d
```

Rebuild and restart services:

```bash
docker compose up -d --build
```

View running containers:

```bash
docker compose ps
```

View backend logs:

```bash
docker compose logs -f backend
```

View all service logs:

```bash
docker compose logs -f
```

Stop all services:

```bash
docker compose down
```

To remove containers and database/cache volumes as well:

```bash
docker compose down -v
```

**Warning:** The `-v` option deletes persistent PostgreSQL and Redis data.

## Project Structure

```text
Webshop/
│
├── backend/
│   ├── migrations/
│   ├── src/
│   │   ├── handlers/
│   │   ├── models/
│   │   ├── repositories/
│   │   ├── routes/
│   │   ├── services/
│   │   ├── auth_middleware.rs
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── Cargo.toml
│   └── Dockerfile
│
├── frontend/
│   ├── app/
│   ├── public/
│   ├── package.json
│   └── Dockerfile
│
├── db/
├── monitoring/
├── docker-compose.yml
├── .gitignore
└── README.md
```

## License

This project was developed as a fullstack software development project.