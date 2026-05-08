# Requirements

## Background

현재 `oci-api`는 OCI 공식 Rust SDK를 사용하는 것이 아니라, OCI REST API를 직접 서명/호출하는 custom Rust client crate다. `Email Delivery`와 `Object Storage`는 이미 소비자 관점에서 SDK처럼 쓸 수 있지만, 인증 방식은 `OCI_USER_ID`, `OCI_FINGERPRINT`, `OCI_PRIVATE_KEY` 등에 의존하는 API key signer로 고정되어 있다.

이 구조는 로컬 개발과 기존 production 연동에는 유효하지만, OCI Compute 위에서 더 안전하게 Vault 기반 secret 관리와 key rotation을 수행하려는 목표에는 한계가 있다. 앱이 Vault에 접근하기 위해 다시 장기 OCI signing private key를 들고 있어야 하기 때문이다. 따라서 `oci-api`는 기존 API key 방식을 유지하면서도, Rust 코드 레벨에서 사용할 수 있는 Instance Principal 인증 계층과 Vault/Keys service SDK surface를 추가해야 한다.

## Project Principles Snapshot

- Source: 2026-05-08 사용자 대화, 현재 `oci-api` 구현 상태
- Principles:
  - 기존 API key 기반 consumer를 깨지 않도록 backward compatibility를 유지한다.
  - auth/provider layer를 분리하여 API key와 Instance Principal을 같은 service client surface 아래에서 교체 가능하게 만든다.
  - downstream consumer는 raw signed HTTP 조립이 아니라 typed Rust SDK surface를 사용하게 한다.
  - OCI 위 런타임에서는 가능하면 장기 OCI signing private key 없이도 동작하는 경로를 제공한다.
  - Vault/Keys/rotation 기능은 testable primitive 단위로 설계하고, consumer rollout은 상위 spec에서 이어받는다.
  - Coolify MCP는 OCI resource를 직접 제어하지 않고, OCI-hosted test runner/container orchestration까지만 담당한다.

## User Stories

### Story 1: crate 유지보수자의 auth foundation 확장

crate 유지보수자로서, 기존 API key signer를 유지하면서도 Instance Principal 인증 provider를 추가하여 OCI 런타임에서 더 안전한 인증 방식을 제공하고 싶다.

### Story 2: 서비스 개발자의 typed SDK 소비

`techton-back` 같은 서비스 개발자로서, raw signed request를 직접 만들지 않고 `oci-api`의 typed service client를 통해 Email Delivery, Object Storage, Vault, Keys를 일관되게 사용하고 싶다.

### Story 3: 운영 담당자의 secret/key rotation 준비

운영 담당자로서, Vault secret 조회와 Key rotation에 필요한 API surface가 Rust crate에 준비되어 있어 consumer 서비스들이 rotation workflow를 구현할 수 있길 원한다.

## Acceptance Criteria

- [ ] AC-1: `oci-api`에 API key와 Instance Principal을 모두 수용하는 auth/provider abstraction이 존재한다.
- [ ] AC-2: `Oci::from_env()` 또는 builder가 auth mode를 선택할 수 있으며, 기존 API key consumer는 호환성을 유지한다.
- [ ] AC-3: 기존 `Email Delivery` / `Object Storage` client가 공통 auth abstraction을 통해 동작하고 회귀 없이 유지된다.
- [ ] AC-4: downstream consumer가 raw request signing을 몰라도 되도록 typed Rust SDK surface가 명확해진다.
- [ ] AC-5: Vault/Secrets client가 current secret bundle 조회와 version/stage 기반 조회 primitive를 제공한다.
- [ ] AC-6: Keys client가 key 조회와 rotate action primitive를 제공한다.
- [ ] AC-7: Instance Principal의 credential/token refresh 전략과 실패 처리 전략이 문서화되고 테스트된다.
- [ ] AC-8: 로컬 개발(API key)과 OCI 런타임(Instance Principal) 양쪽 사용 예시와 검증 전략이 문서화된다.
- [ ] AC-9: 실제 OCI instance(Coolify 경유 포함)에서 test resource를 사용한 smoke 검증 경로가 정의된다.
- [ ] AC-10: Coolify MCP와 `oci-api`/OCI API 계층의 경계가 문서와 작업 항목에 반영된다.

## Scope

### Included

- auth/provider abstraction 도입
- API key signer를 감싼 compatibility auth provider
- Instance Principal auth provider 및 refresh/cache 전략
- `Email Delivery` / `Object Storage`의 공통 auth layer 전환
- `Vault/Secrets` service client 추가
- `Keys` service client 및 rotate primitive 추가
- 문서/예제/테스트 갱신
- OCI-hosted smoke에 필요한 test resource/fixture 요구사항 정의

### Excluded

- `techton-back`, `techton-front` consumer migration 자체
- 브라우저 런타임에서의 Vault direct access
- 주기 스케줄러/운영 자동화까지 포함한 end-to-end rotation orchestration
- OCI IAM dynamic group / policy 생성 자동화
- 실제 test resource provisioning 자동화

## References


| Relation | Spec | Note |
|----------|------|------|
| Parent   | `../../../../../techton/.github/specs/2605080457Z_OCI-Vault-Credential-Platform` | Phase 1 of 2 |
