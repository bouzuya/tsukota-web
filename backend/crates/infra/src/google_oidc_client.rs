use application::AuthorizationRequest;
use application::OidcClaims;
use application::OidcClient;
use application::OidcError;
use async_trait::async_trait;
use openidconnect::AuthorizationCode;
use openidconnect::ClientId;
use openidconnect::ClientSecret;
use openidconnect::CsrfToken;
use openidconnect::IssuerUrl;
use openidconnect::Nonce;
use openidconnect::PkceCodeChallenge;
use openidconnect::PkceCodeVerifier;
use openidconnect::RedirectUrl;
use openidconnect::core::CoreAuthenticationFlow;
use openidconnect::core::CoreClient;
use openidconnect::core::CoreProviderMetadata;

/// 起動時に discover した Google OIDC メタデータを持つ `openidconnect::Client` の型エイリアス
///
/// 型パラメータが多いため alias で隠蔽する
type ConfiguredCoreClient = openidconnect::Client<
    openidconnect::EmptyAdditionalClaims,
    openidconnect::core::CoreAuthDisplay,
    openidconnect::core::CoreGenderClaim,
    openidconnect::core::CoreJweContentEncryptionAlgorithm,
    openidconnect::core::CoreJsonWebKey,
    openidconnect::core::CoreAuthPrompt,
    openidconnect::StandardErrorResponse<openidconnect::core::CoreErrorResponseType>,
    openidconnect::core::CoreTokenResponse,
    openidconnect::core::CoreTokenIntrospectionResponse,
    openidconnect::core::CoreRevocableToken,
    openidconnect::core::CoreRevocationErrorResponse,
    openidconnect::EndpointSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointMaybeSet,
    openidconnect::EndpointMaybeSet,
>;

/// Google を IdP とする OIDC クライアントの実装
///
/// 起動時に discover した OIDC メタデータを保持し、authorize_url 生成と
/// authorization code → id_token の交換 + 検証を行う
#[derive(Clone)]
pub struct GoogleOidcClient {
    client: ConfiguredCoreClient,
    http_client: reqwest::Client,
}

impl GoogleOidcClient {
    /// OIDC discover を行い、Google 用の CoreClient を構築する
    pub async fn discover(
        issuer_url: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
    ) -> Result<Self, OidcError> {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        let issuer = IssuerUrl::new(issuer_url.to_owned())?;
        let provider_metadata = CoreProviderMetadata::discover_async(issuer, &http_client).await?;
        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(client_id.to_owned()),
            Some(ClientSecret::new(client_secret.to_owned())),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri.to_owned())?);
        Ok(Self {
            client,
            http_client,
        })
    }
}

#[async_trait]
impl OidcClient for GoogleOidcClient {
    fn authorize_url(&self) -> Result<AuthorizationRequest, OidcError> {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        // openidconnect crate が `openid` scope を自動付与するので明示追加はしない
        let (auth_url, state, nonce) = self
            .client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(challenge)
            .url();
        Ok(AuthorizationRequest {
            authorize_url: auth_url.to_string(),
            state: state.secret().clone(),
            nonce: nonce.secret().clone(),
            pkce_verifier: verifier.secret().clone(),
        })
    }

    async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: &str,
        nonce: &str,
    ) -> Result<OidcClaims, OidcError> {
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_owned()))?
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_owned()))
            .request_async(&self.http_client)
            .await?;
        let id_token = openidconnect::TokenResponse::id_token(&token_response)
            .ok_or_else(|| -> OidcError { "No ID token in response".into() })?;
        let id_token_verifier = self.client.id_token_verifier();
        let nonce = Nonce::new(nonce.to_owned());
        let claims = id_token.claims(&id_token_verifier, &nonce)?;
        Ok(OidcClaims {
            sub: claims.subject().to_string(),
        })
    }
}
