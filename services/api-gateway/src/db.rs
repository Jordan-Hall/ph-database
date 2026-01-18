use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

#[derive(Clone)]
pub struct Database {
    pub client: Surreal<Client>,
}

impl Database {
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

    /// Execute a query with parameters
    pub async fn query<T>(&self, sql: &str) -> Result<Vec<T>, surrealdb::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut result = self.client.query(sql).await?;
        result.take(0)
    }

    /// Create a record in a table
    pub async fn create<T>(&self, table: &str, data: T) -> Result<Vec<T>, surrealdb::Error>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        self.client.create(table).content(data).await
    }

    /// Select a specific record
    pub async fn select<T>(&self, thing: &str) -> Result<Option<T>, surrealdb::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        self.client.select(thing).await
    }

    /// Update a record
    pub async fn update<T>(&self, thing: &str, data: T) -> Result<Option<T>, surrealdb::Error>
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        self.client.update(thing).content(data).await
    }

    /// Delete a record
    pub async fn delete<T>(&self, thing: &str) -> Result<Option<T>, surrealdb::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        self.client.delete(thing).await
    }
}
