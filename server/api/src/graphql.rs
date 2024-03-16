use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    response::{self, IntoResponse},
    Extension,
};

pub async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

// root query
pub struct Query;

#[Object]
impl Query {
    async fn hello(&self) -> &'static str {
        "🌍"
    }
}

pub type GraphQLSchema = Schema<Query, EmptyMutation, EmptySubscription>;

// build schema and write (req independent) state to it
pub fn build_schema() -> GraphQLSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}

// add req based data to the context
pub async fn graphql_handler(
    schema: Extension<GraphQLSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let req = req.into_inner();
    schema.execute(req).await.into()
}
