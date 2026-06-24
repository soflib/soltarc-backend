use anyhow::Result;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};

use crate::generated::auth::{
    auth_service_client::AuthServiceClient,
    AuthResponse,
    RegisterRequest,
    LoginRequest,
    LogoutRequest,         LogoutResponse,
    RefreshRequest,
    ValidateRequest,       ValidateResponse,
    RevokeSessionsRequest, RevokeSessionsResponse,
    ChangePasswordRequest, ChangePasswordResponse,
    RequestPasswordResetRequest, RequestPasswordResetResponse,
    ConfirmPasswordResetRequest, ConfirmPasswordResetResponse,
    GetAllUsersRequest,    GetAllUsersResponse,
    GetUserRequest,        UserDetailResponse,
    DeleteUserRequest,     DeleteUserResponse,
    UpdateUserRequest,
    LockUserRequest,       LockUserResponse,
    CreateTenantRequest,   GetTenantRequest, UpdateTenantRequest,
    DeleteTenantRequest,   DeleteTenantResponse,
    ListTenantsRequest,    ListTenantsResponse, TenantResponse,
    SetTenantDbUrlRequest, SetTenantDbUrlResponse,
    GetTenantDbUrlRequest, GetTenantDbUrlResponse,
};

#[derive(Clone)]
pub struct AuthGrpcClient {
    inner: AuthServiceClient<Channel>,
}

impl AuthGrpcClient {
    pub async fn connect_insecure(addr: &str) -> Result<Self> {
        let channel = Channel::from_shared(addr.to_string())?
            .connect()
            .await?;
        Ok(Self { inner: AuthServiceClient::new(channel) })
    }

    pub async fn connect_mtls(
        addr:        &str,
        domain_name: &str,
        ca_cert:     &[u8],
        client_cert: &[u8],
        client_key:  &[u8],
    ) -> Result<Self> {
        let tls = ClientTlsConfig::new()
            .domain_name(domain_name)
            .ca_certificate(Certificate::from_pem(ca_cert))
            .identity(Identity::from_pem(client_cert, client_key));

        let channel = Channel::from_shared(addr.to_string())?
            .tls_config(tls)?
            .connect()
            .await?;

        Ok(Self { inner: AuthServiceClient::new(channel) })
    }

    // ── Identity ─────────────────────────────────────────────────────────────

    pub async fn register(&mut self, req: RegisterRequest) -> Result<AuthResponse, tonic::Status> {
        self.inner.register(req).await.map(|r| r.into_inner())
    }

    pub async fn login(&mut self, req: LoginRequest) -> Result<AuthResponse, tonic::Status> {
        self.inner.login(req).await.map(|r| r.into_inner())
    }

    pub async fn logout(&mut self, req: LogoutRequest) -> Result<LogoutResponse, tonic::Status> {
        self.inner.logout(req).await.map(|r| r.into_inner())
    }

    // ── Tokens ───────────────────────────────────────────────────────────────

    pub async fn refresh_token(&mut self, req: RefreshRequest) -> Result<AuthResponse, tonic::Status> {
        self.inner.refresh_token(req).await.map(|r| r.into_inner())
    }

    pub async fn validate_token(&mut self, req: ValidateRequest) -> Result<ValidateResponse, tonic::Status> {
        self.inner.validate_token(req).await.map(|r| r.into_inner())
    }

    // ── Sessions ─────────────────────────────────────────────────────────────

    pub async fn revoke_sessions(&mut self, req: RevokeSessionsRequest) -> Result<RevokeSessionsResponse, tonic::Status> {
        self.inner.revoke_sessions(req).await.map(|r| r.into_inner())
    }

    // ── Credentials ──────────────────────────────────────────────────────────

    pub async fn change_password(&mut self, req: ChangePasswordRequest) -> Result<ChangePasswordResponse, tonic::Status> {
        self.inner.change_password(req).await.map(|r| r.into_inner())
    }

    // ── Password reset (forgot password) ───────────────────────────────────────

    pub async fn request_password_reset(&mut self, req: RequestPasswordResetRequest) -> Result<RequestPasswordResetResponse, tonic::Status> {
        self.inner.request_password_reset(req).await.map(|r| r.into_inner())
    }

    pub async fn confirm_password_reset(&mut self, req: ConfirmPasswordResetRequest) -> Result<ConfirmPasswordResetResponse, tonic::Status> {
        self.inner.confirm_password_reset(req).await.map(|r| r.into_inner())
    }

    // ── Users ────────────────────────────────────────────────────────────────

    pub async fn get_all_users(&mut self, req: GetAllUsersRequest) -> Result<GetAllUsersResponse, tonic::Status> {
        self.inner.get_all_users(req).await.map(|r| r.into_inner())
    }

    pub async fn get_user(&mut self, req: GetUserRequest) -> Result<UserDetailResponse, tonic::Status> {
        self.inner.get_user(req).await.map(|r| r.into_inner())
    }

    pub async fn delete_user(&mut self, req: DeleteUserRequest) -> Result<DeleteUserResponse, tonic::Status> {
        self.inner.delete_user(req).await.map(|r| r.into_inner())
    }

    pub async fn update_user(&mut self, req: UpdateUserRequest) -> Result<UserDetailResponse, tonic::Status> {
        self.inner.update_user(req).await.map(|r| r.into_inner())
    }

    pub async fn lock_user(&mut self, req: LockUserRequest) -> Result<LockUserResponse, tonic::Status> {
        self.inner.lock_user(req).await.map(|r| r.into_inner())
    }

    // ── Tenants ──────────────────────────────────────────────────────────────

    pub async fn create_tenant(&mut self, req: CreateTenantRequest) -> Result<TenantResponse, tonic::Status> {
        self.inner.create_tenant(req).await.map(|r| r.into_inner())
    }

    pub async fn get_tenant(&mut self, req: GetTenantRequest) -> Result<TenantResponse, tonic::Status> {
        self.inner.get_tenant(req).await.map(|r| r.into_inner())
    }

    pub async fn update_tenant(&mut self, req: UpdateTenantRequest) -> Result<TenantResponse, tonic::Status> {
        self.inner.update_tenant(req).await.map(|r| r.into_inner())
    }

    pub async fn delete_tenant(&mut self, req: DeleteTenantRequest) -> Result<DeleteTenantResponse, tonic::Status> {
        self.inner.delete_tenant(req).await.map(|r| r.into_inner())
    }

    pub async fn list_tenants(&mut self, req: ListTenantsRequest) -> Result<ListTenantsResponse, tonic::Status> {
        self.inner.list_tenants(req).await.map(|r| r.into_inner())
    }

    // ── Tenant Secrets ───────────────────────────────────────────────────────

    pub async fn set_tenant_db_url(&mut self, req: SetTenantDbUrlRequest) -> Result<SetTenantDbUrlResponse, tonic::Status> {
        self.inner.set_tenant_db_url(req).await.map(|r| r.into_inner())
    }

    pub async fn get_tenant_db_url(&mut self, req: GetTenantDbUrlRequest) -> Result<GetTenantDbUrlResponse, tonic::Status> {
        self.inner.get_tenant_db_url(req).await.map(|r| r.into_inner())
    }
}
