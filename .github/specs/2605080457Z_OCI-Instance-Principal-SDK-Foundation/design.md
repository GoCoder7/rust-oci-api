# Design

## Design Goals

- 기존 API key auth 경로를 유지하면서 Instance Principal을 추가한다.
- service client가 auth 구현 세부를 몰라도 되도록 공통 auth/provider 계층을 도입한다.
- Email/Object Storage 외에 Vault/Keys를 같은 Rust SDK surface 아래에서 제공한다.
- token/credential refresh와 rotation primitive를 테스트 가능한 구조로 설계한다.

## Non-Goals

- consumer 프로젝트(`techton-back`, `techton-front`) 코드 변경
- OCI IAM dynamic group / policy 생성 스크립트 작성
- 브라우저용 client SDK 제공
- 주기 실행 스케줄러까지 포함한 완전 자동 rotation orchestration

## Architecture

### Proposed module shape

```text
src/
├── auth/
│   ├── config_loader.rs
│   ├── key_loader.rs
│   └── providers/
│       ├── api_key.rs
│       ├── instance_principal.rs
│       └── mod.rs
├── client/
│   ├── http.rs
│   ├── request_executor.rs
│   └── signer.rs
└── services/
    ├── email/
    ├── object_storage/
    ├── vault/
    └── keys/
```

### Auth abstraction

`Oci`는 더 이상 API key signer에만 결합되지 않고, 공통 auth/provider 인터페이스를 통해 요청을 인증한다.

```rust
#[async_trait]
pub trait OciAuthProvider: Send + Sync {
    async fn authorize(
        &self,
        method: &str,
        path: &str,
        host: &str,
        body: Option<&str>,
    ) -> Result<Vec<(String, String)>>;
}
```

- `ApiKeyAuthProvider`
  - 현재 `OciSigner`를 감싸서 기존 `tenancy/user/fingerprint + private_key` 방식 유지
  - `Oci::from_env()`의 기본/호환 경로
- `InstancePrincipalAuthProvider`
  - OCI instance metadata / credentials service에서 instance principal용 auth state를 확보
  - 만료 전 refresh 가능한 cache를 보유
  - local dev에서는 사용하지 않고 OCI runtime에서만 활성화

### Service client composition

`Email Delivery`, `Object Storage`, `Vault`, `Keys`는 공통 request executor를 통해 auth provider를 사용한다.

```text
Service client
└── RequestExecutor
    ├── build request
    ├── ask auth provider for headers/auth state
    └── execute via reqwest::Client
```

이 구조를 통해 service 구현은 "무슨 endpoint를 호출하는가"에 집중하고, "어떻게 인증하는가"는 provider 계층으로 이동한다.

### External control boundary

```text
Coolify MCP
└── OCI-hosted test runner/container lifecycle only

`oci-api` / OCI API layer
└── Real OCI resource operations
    ├── auth via API key or Instance Principal
    ├── Vault secret operations
    └── Keys rotation operations
```

## Data Flow

1. Consumer가 `Oci::from_env()` 또는 `Oci::builder()`를 통해 auth mode를 선택한다.
2. `ApiKey` 모드에서는 기존 signer 기반 header 생성이 유지된다.
3. `Instance Principal` 모드에서는 provider가 OCI instance metadata/credentials 흐름을 통해 auth state를 확보하고 cache한다.
4. 각 service client는 공통 executor를 사용해 요청을 만들고, provider가 생성한 인증 정보와 함께 OCI REST API를 호출한다.
5. `Vault`는 secret bundle current/version/stage 조회를 제공하고, `Keys`는 key 조회/rotate primitive를 제공한다.
6. consumer는 반환된 typed model을 사용해 secret 조회, key rotation workflow, 기존 email/object storage 호출을 처리한다.
7. 실제 end-to-end confidence는 OCI instance/Coolify 환경에서 test resource를 사용하는 smoke step으로 확보한다.

## Constraints

- 현재 crate는 API key signer와 환경변수(`OCI_USER_ID`, `OCI_TENANCY_ID`, `OCI_REGION`, `OCI_FINGERPRINT`, `OCI_PRIVATE_KEY`, `OCI_COMPARTMENT_ID`)를 전제로 한다.
- local dev/test에서 Instance Principal은 사용할 수 없으므로 API key path와 mockable test path가 계속 필요하다.
- OCI 쪽에는 현재 이 프로젝트가 바로 가져다 쓸 Rust SDK가 없으므로, crate는 계속 custom REST wrapper + SDK surface 형태를 유지한다.
- Instance Principal은 OCI Compute 환경과 적절한 IAM policy/dynamic group이 있어야 실제 검증 가능하다.
- published API를 깨지 않으려면 `email_delivery()`와 `object_storage()` 호출자 입장에서의 breaking change를 최소화해야 한다.
- 실제 Vault/Keys smoke test를 위해서는 테스트용 secret/key/resource naming과 정리 정책을 먼저 정해야 한다.

## Technical Decisions

1. **Compatibility default**: 기존 consumer가 깨지지 않게 API key auth를 기본 호환 경로로 유지한다.
2. **Explicit auth mode**: `OCI_AUTH_MODE=api_key|instance_principal` 같은 명시적 선택지 또는 builder 옵션을 제공한다.
3. **Shared executor**: service별로 중복된 reqwest/header 조립을 줄이기 위해 공통 request executor를 도입한다.
4. **Provider-owned refresh**: Instance Principal의 token/credential refresh 책임은 provider 내부에 둔다.
5. **Rotation as primitive**: 이번 phase에서는 secret version 조회와 key rotate action 같은 기본 primitive를 제공하고, consumer orchestration은 후속 phase로 넘긴다.
6. **Layered validation**: 로컬 unit/mocked integration으로 대부분 개발하고, 마지막에는 OCI-hosted smoke로 실제 auth/resource 권한을 검증한다.
7. **Boundary clarity**: Coolify MCP는 test runner orchestration만 담당하고, 실제 Vault/Keys 조작은 `oci-api` 또는 다른 OCI API client가 담당한다.
8. **Auth mode default**: 구현 기본값은 `OCI_AUTH_MODE=api_key|instance_principal`이며, builder에서도 같은 선택지를 제공한다.
9. **Provider interface shape**: provider는 요청별 인증 헤더/인증 상태를 만들어 주고, service client는 공통 executor만 의존한다.
10. **Phase-1 Vault/Keys scope lock**: Vault는 current secret + version/stage 조회, Keys는 key 조회 + rotate action까지만 1차 범위로 고정한다.
11. **Test resource policy**: smoke용 secret/key/resource는 전용 prefix/naming을 사용하고 production/shared resource를 절대 재사용하지 않는다.
12. **Cleanup policy**: smoke 테스트 후 정리 절차와 실패 시 수동 정리 목록을 반드시 남긴다.

## Validation Strategy

1. API key auth 경로의 기존 email/object storage regression test를 유지한다.
2. mocked metadata/token 환경에서 Instance Principal provider의 refresh/expiry/failure 경로를 테스트한다.
3. Vault/Keys client에 대해 endpoint/model/unit test를 추가한다.
4. public examples와 README가 API key local dev, OCI runtime usage를 모두 설명하는지 검증한다.
5. 필요 시 실제 OCI 환경에서는 manual smoke test로 Instance Principal + Vault/Keys 동작을 확인한다.
6. OCI-hosted smoke test는 test secret, test key, 필요 시 별도 test compartment/resource naming 규칙을 사용하고, 테스트 후 정리 책임을 문서화한다.
7. 문서와 tasks가 위 기본값(`OCI_AUTH_MODE`, phase-1 endpoint 범위, test resource/cleanup 정책)과 일치하는지 검토한다.
