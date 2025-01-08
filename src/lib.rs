use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemImpl, Meta, NestedMeta};

/// An attribute macro enabling a structured approach to defining routes and attaching middleware for axum servers
///
/// This macro generates an axum router for a struct by combining annotated methods into routes with specified HTTP methods and paths.
/// It also allows applying middleware functions to the routes.
///
/// # Arguments
///
/// - `path`: Specifies a base path for all routes within the controller.
/// - `middleware`: A list of middleware functions to apply to all routes in the controller.
///
/// # Simple Web Server with AppState
///
/// ```rust
/// use axum::{
///     body::Body,
///     extract::{Request, State},
///     http::StatusCode,
///     middleware::Next,
///     response::{IntoResponse, Response},
///     Json, Router,
/// };
/// use route_controller::controller;
/// use route_controller::route;
///
/// #[derive(Clone)]
/// pub struct AppState {}
///
/// pub struct ApiController;
///
/// #[controller(
///     path = "/api",
///     middleware = "auth_middleware",
///     middleware = "context_middleware"
/// )]
/// impl ApiController {
///     #[route("GET", "/users")]
///     pub async fn get_users(_request: Request) -> impl IntoResponse {
///         Json(vec!["user1", "user2"])
///     }
///
///     #[route("POST", "/users")]
///     pub async fn create_user(
///         State(_app_state): State<AppState>,
///         _request: Request
///     ) -> impl IntoResponse {
///         Json("User Created")
///     }
/// }
///
/// async fn auth_middleware(
///     State(_app_state): State<AppState>,
///     req: Request,
///     next: Next,
/// ) -> Result<Response<Body>, StatusCode> {
///     Ok(next.run(req).await)
/// }
///
/// async fn context_middleware(mut req: Request, next: Next) -> Result<Response<Body>, StatusCode> {
///     let trace_id = "123"; // Replace with a uuid
///     req.headers_mut()
///         .append("trace_id", trace_id.parse().unwrap());
///     Ok(next.run(req).await)
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let app_state = AppState {};
///     let router = Router::new()
///         .merge(ApiController::router(app_state.clone()))
///         .with_state(app_state);
///     let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
///     axum::serve(listener, router).await.unwrap();
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates a `router` function that constructs an `axum::Router` by combining the specified routes and applying the middleware.
///
/// Each method annotated with `#[route]` is mapped to its respective HTTP method and path. The router is then available to be mounted or used directly in an Axum application.
#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attr as syn::AttributeArgs);

    let mut base_path = None;
    let mut middlewares = Vec::new();

    for arg in attr_args.iter() {
        if let NestedMeta::Meta(Meta::NameValue(nv)) = arg {
            if nv.path.is_ident("path") {
                if let syn::Lit::Str(lit_str) = &nv.lit {
                    base_path = Some(lit_str.value());
                }
            } else if nv.path.is_ident("middleware") {
                if let syn::Lit::Str(lit_str) = &nv.lit {
                    middlewares.push(lit_str.value());
                }
            }
        }
    }

    let ast = parse_macro_input!(item as ItemImpl);
    let struct_name = &ast.self_ty;

    let mut routes: Vec<(String, String, String)> = Vec::new();

    for item in ast.items.iter() {
        if let syn::ImplItem::Method(method) = item {
            for attr in &method.attrs {
                if attr.path.is_ident("route") {
                    if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                        let mut method_str = None;
                        let mut path_str = None;

                        for nested in meta_list.nested.iter() {
                            if let NestedMeta::Lit(syn::Lit::Str(lit_str)) = nested {
                                if method_str.is_none() {
                                    method_str = Some(lit_str.value());
                                } else if path_str.is_none() {
                                    path_str = Some(lit_str.value());
                                }
                            }
                        }

                        let method_str =
                            method_str.expect("Expected a string literal for HTTP method");
                        let path_str = path_str.expect("Expected a string literal for path");

                        routes.push((method_str, path_str, method.sig.ident.to_string()));
                    }
                }
            }
        }
    }

    let route_definitions = routes.iter().map(|(method, path, handler_name)| {
        let full_path = format!("{}{}", base_path.as_deref().unwrap_or(""), path);
        let handler_ident = format_ident!("{}", handler_name);

        match method.as_str() {
            "DELETE" => quote! { .route(#full_path, axum::routing::delete(Self::#handler_ident)) },
            "GET" => quote! { .route(#full_path, axum::routing::get(Self::#handler_ident)) },
            "HEAD" => quote! { .route(#full_path, axum::routing::head(Self::#handler_ident)) },
            "OPTIONS" => {
                quote! { .route(#full_path, axum::routing::options(Self::#handler_ident)) }
            }
            "PATCH" => quote! { .route(#full_path, axum::routing::patch(Self::#handler_ident)) },
            "POST" => quote! { .route(#full_path, axum::routing::post(Self::#handler_ident)) },
            "PUT" => quote! { .route(#full_path, axum::routing::put(Self::#handler_ident)) },
            "TRACE" => quote! { .route(#full_path, axum::routing::trace(Self::#handler_ident)) },
            _ => quote! {},
        }
    });

    let middleware_stack = middlewares.iter().map(|middleware| {
        let middleware_ident = format_ident!("{}", middleware);
        quote! { .layer(axum::middleware::from_fn_with_state(state.clone(), #middleware_ident)) }
    });

    let expanded = quote! {
        #ast

        impl #struct_name {
            pub fn router(state: AppState) -> axum::Router<AppState> {
                axum::Router::new()
                    #(#route_definitions)*
                    #(#middleware_stack)*
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn route(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
