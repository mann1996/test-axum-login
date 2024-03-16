use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum User {
    #[sea_orm(iden = "users")]
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Provider {
    #[sea_orm(iden = "providers")]
    Table,
    Id,
    ProviderId,
    Name,
    CreatedAt,
    UserId,
    AccessToken,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Provider::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Provider::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Provider::Name).string().not_null())
                    .col(ColumnDef::new(Provider::ProviderId).string().not_null())
                    .col(ColumnDef::new(Provider::AccessToken).string().not_null())
                    .col(ColumnDef::new(Provider::UserId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Provider::Table, Provider::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(Provider::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Provider::Table).to_owned())
            .await
    }
}
