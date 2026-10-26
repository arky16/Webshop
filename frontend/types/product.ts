export interface Product{
    id: string;
    name: string;
    description: string;
    price: string;
    stock: number;
    category: string | null;
    active: boolean;
    created_at: string;
    updated_at: string;
}

export interface ProductPagination{
    page: number;
    size: number;
    total: number;
    total_pages:number;
}

export interface ProductListResponse{
    products: Product[];
    pagination: ProductPagination;
}