use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Value,
    Surreal,
};
use serde_json::json;

#[derive(Clone)]
pub struct Database {
    pub client: Surreal<Client>,
}

/// Response from SurrealDB authentication
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AuthToken {
    pub token: String,
}

impl Database {
    /// Create a new database connection with root access
    pub async fn new(url: &str) -> Result<Self, surrealdb::Error> {
        let client = Surreal::new::<Ws>(url).await?;

        // Sign in as root user
        client
            .signin(Root {
                username: "root",
                password: "root",
            })
            .await?;

        // Use namespace and database
        client.use_ns("prod").use_db("main").await?;

        Ok(Self { client })
    }

    /// Authenticate user with SurrealDB scope (SIGNUP)
    pub async fn signup(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<String, surrealdb::Error> {
        let mut result = self.client
            .query("SIGNUP { scope: 'user_scope', username: $username, email: $email, password: $password }")
            .bind(("username", username))
            .bind(("email", email))
            .bind(("password", password))
            .await?;

        let token: Option<String> = result.take(0)?;
        token.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::Query("Signup failed".to_string())))
    }

    /// Authenticate user with SurrealDB scope (SIGNIN)
    pub async fn signin(
        &self,
        email: String,
        password: String,
    ) -> Result<String, surrealdb::Error> {
        let mut result = self.client
            .query("SIGNIN { scope: 'user_scope', email: $email, password: $password }")
            .bind(("email", email))
            .bind(("password", password))
            .await?;

        let token: Option<String> = result.take(0)?;
        token.ok_or_else(|| surrealdb::Error::Api(surrealdb::error::Api::Query("Signin failed".to_string())))
    }

    /// Verify and decode a SurrealDB token by creating a temporary connection
    /// Returns the authenticated user data from the token
    pub async fn verify_token(&self, token: &str, db_url: &str) -> Result<serde_json::Value, surrealdb::Error> {
        // Create a temporary client to verify the token
        let temp_client = Surreal::new::<Ws>(db_url).await?;

        // Authenticate using the provided token
        temp_client.authenticate(token).await?;

        // Use the same namespace and database
        temp_client.use_ns("prod").use_db("main").await?;

        // Query the authenticated user context
        let mut result = temp_client
            .query("SELECT * FROM $auth")
            .await?;

        let user_data: Option<serde_json::Value> = result.take(0)?;

        user_data.ok_or_else(|| {
            surrealdb::Error::Api(surrealdb::error::Api::Query(
                "Token verification failed: no auth context".to_string()
            ))
        })
    }

    /// Execute a query with parameters
    pub async fn query<T>(&self, sql: &str) -> Result<Vec<T>, surrealdb::Error>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let mut result = self.client.query(sql).await?;
        result.take(0)
    }

    /// Execute a query with a single bind variable
    pub async fn query_bind<T>(
        &self,
        sql: &str,
        name: &'static str,
        value: impl serde::Serialize + 'static,
    ) -> Result<Vec<T>, surrealdb::Error>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        let mut result = self.client.query(sql).bind((name, value)).await?;
        result.take(0)
    }

    /// Execute a parameterized query with multiple bindings
    pub async fn query_with_params<T>(
        &self,
        sql: &str,
        _params: serde_json::Value,
    ) -> Result<Vec<T>, surrealdb::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        // For now, use a simpler approach with raw query
        // TODO: Implement proper binding for multiple params
        let mut result = self.client.query(sql).await?;
        result.take(0)
    }

    /// Create a record in a table
    pub async fn create<T>(&self, table: &str, data: T) -> Result<Option<T>, surrealdb::Error>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + 'static,
    {
        self.client.create(table).content(data).await
    }

    /// Select a specific record
    pub async fn select<T>(&self, thing: &str) -> Result<Vec<T>, surrealdb::Error>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        self.client.select(thing).await
    }

    /// Update a record
    pub async fn update<T>(&self, thing: &str, data: T) -> Result<Vec<T>, surrealdb::Error>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + 'static,
    {
        self.client.update(thing).content(data).await
    }

    /// Delete a record
    pub async fn delete<T>(&self, thing: &str) -> Result<Vec<T>, surrealdb::Error>
    where
        T: serde::de::DeserializeOwned + 'static,
    {
        self.client.delete(thing).await
    }
}
