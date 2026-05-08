# Task Plan

## Task Principles

- Follow Red -> Green -> Refactor whenever possible.
- Do not start the next phase until the current phase is verified.
- Record actual execution results, not just intentions.

## Preparation

- [ ] 기존 API key auth 경로와 public API surface(`Oci::from_env`, `email_delivery`, `object_storage`)의 baseline을 기록한다.
- [ ] Instance Principal 검증에 필요한 metadata/token mock 전략과 실제 OCI smoke 조건을 정리한다.
- [ ] Vault/Keys endpoint 범위(current secret, staged/versioned secret, key read, rotate)를 확정한다.
- [ ] OCI-hosted smoke에 필요한 test secret/key/resource naming과 cleanup 규칙을 정리한다.
- [ ] Coolify MCP는 test-runner orchestration, `oci-api`는 실제 OCI resource 호출을 담당한다는 경계를 명시한다.
- [ ] 권장 기본값(`OCI_AUTH_MODE`, provider shape, phase-1 endpoint scope, test resource/cleanup policy)을 구현 기준으로 잠근다.

## Step 1: auth/provider abstraction 도입

- Covers: AC-1, AC-2, AC-3, AC-4
- [ ] `Red`: 현재 signer 고정 구조에서 provider abstraction을 도입할 때 깨지는 컴파일/테스트 지점을 확인한다.
- [ ] `Green`: 공통 auth/provider trait과 API key compatibility provider를 추가한다.
- [ ] `Green`: `Oci::from_env()` / builder가 auth mode를 선택할 수 있게 확장한다.
- [ ] `Refactor`: `Oci` 내부 상태와 service client 생성 경로를 provider 중심으로 정리한다.

## Step 2: Instance Principal provider 구현

- Covers: AC-1, AC-2, AC-7, AC-8, AC-9
- [ ] `Red`: metadata/token fetch, expiry, refresh failure에 대한 테스트를 먼저 작성한다.
- [ ] `Green`: Instance Principal provider와 cache/refresh 로직을 구현한다.
- [ ] `Green`: OCI runtime 전용 설정/예시를 문서화한다.
- [ ] `Refactor`: local dev와 OCI runtime 분기 규칙을 단순화한다.

## Step 3: 기존 service client를 공통 SDK 계층으로 이관

- Covers: AC-3, AC-4
- [ ] `Red`: Email Delivery / Object Storage가 provider abstraction 아래에서도 기존 호출자와 호환되는지 regression test를 작성한다.
- [ ] `Green`: 중복된 request/header 조립을 공통 executor로 이동한다.
- [ ] `Green`: `email_delivery()`와 `object_storage()` 소비자 API를 가능한 한 유지한다.
- [ ] `Refactor`: raw signing 세부가 public 사용 예제에 드러나지 않도록 정리한다.

## Step 4: Vault/Secrets client 추가

- Covers: AC-4, AC-5, AC-8
- [ ] `Red`: current secret bundle 조회와 version/stage 조회 테스트를 작성한다.
- [ ] `Green`: Vault/Secrets client와 typed models를 구현한다.
- [ ] `Refactor`: secret decoding, metadata 모델, 에러 표면을 정리한다.

## Step 5: Keys client 및 rotation primitive 추가

- Covers: AC-4, AC-6, AC-8
- [ ] `Red`: key 조회와 rotate action 테스트를 작성한다.
- [ ] `Green`: Keys client와 rotate primitive를 구현한다.
- [ ] `Refactor`: rotation 결과 모델과 consumer-facing method naming을 정리한다.

## Step 6: 문서/예제/검증 마무리

- Covers: AC-2, AC-7, AC-8, AC-9, AC-10
- [ ] `Red`: README/examples가 auth mode/Vault/Keys를 설명하지 못하는 부분을 식별한다.
- [ ] `Green`: local API key 개발 경로와 OCI Instance Principal 운영 경로 예제를 추가한다.
- [ ] `Green`: OCI instance/Coolify 경유 test resource smoke 체크 실행 가이드를 정리한다.
- [ ] `Green`: test secret/key/resource cleanup 절차를 문서화한다.
- [ ] `Green`: Coolify MCP와 OCI API 계층의 책임 경계를 문서화한다.
- [ ] `Refactor`: 공개 API naming과 module export를 재정리한다.

## Completion Criteria

- [ ] API key와 Instance Principal을 모두 다룰 수 있는 auth/provider layer가 정의되어 있다.
- [ ] 기존 Email Delivery / Object Storage consumer가 compatibility를 유지한다.
- [ ] Vault/Secrets와 Keys client가 current target use case를 커버한다.
- [ ] rotation primitive 범위와 후속 consumer rollout handoff가 명확하다.
- [ ] README/examples/tests가 두 auth mode와 새 service surface를 설명한다.
- [ ] OCI-hosted smoke test 경로와 test resource 정리 절차가 포함되어 있다.
- [ ] Coolify MCP와 OCI API 계층의 역할 경계가 명확하다.

## Progress Log

### Preparation Results

- 현재 crate는 API key signer 기반 custom REST client이며, Instance Principal 구현은 비어 있다는 점을 기준 baseline으로 삼았다.
- parent spec은 `techton`에 두고, 이 spec은 Phase 1 sub spec으로 위치시켰다.
- 구현 목표를 auth foundation → existing services migration → Vault/Keys → rotation primitive 순으로 분해했다.
- 실제 Instance Principal/Vault/Keys 신뢰 검증은 OCI instance(Coolify 경유 포함)에서 test resource를 사용한 smoke 단계가 필요하다는 점을 추가 반영했다.
- Coolify MCP는 OCI-hosted test runner/container orchestration에만 사용하고, 실제 OCI 리소스 조작은 `oci-api`가 맡는다는 경계를 추가했다.
- 사용자 승인에 따라 `OCI_AUTH_MODE`, provider shape, phase-1 endpoint 범위, test resource/cleanup 정책을 권장 기본값으로 고정했다.

### Step 1 Results

- `Oci` now selects auth mode from `OCI_AUTH_MODE=api_key|instance_principal` while keeping `Oci::from_env()` as the public sync entrypoint.
- API key signer wiring was wrapped behind an auth provider abstraction so shared request execution no longer depends on service-local signing code.
- Shared request execution now owns canonical signed header assembly and request dispatch for migrated services.

### Step 2 Results

- `InstancePrincipalAuthProvider` now fetches IMDS metadata, signs the federation request, caches the OCI security token/session signer, and refreshes before expiry.
- Mocked tests cover IMDS region lookup and a metadata + federation + token reuse path.
- Local mock coverage exists, but OCI-hosted smoke validation is still required for real Instance Principal behavior.

### Step 3 Results

- Email Delivery and Object Storage now use the shared executor instead of direct signer orchestration in service code.
- Object Storage upload retained checksum header support while moving to the executor path.
- Existing library tests for Email/Object Storage continue to pass after the migration.

### Step 4 Results

- Added a Phase 1 Vault Secrets client with current, staged, and versioned secret bundle reads plus typed bundle decoding helpers.
- Dedicated Vault regression tests are still pending.

### Step 5 Results

- Added a Phase 1 Keys client with key lookup and rotate action primitives.
- Dedicated Keys regression tests are still pending.
