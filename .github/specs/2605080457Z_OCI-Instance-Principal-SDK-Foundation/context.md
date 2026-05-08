# Context

This document records the conversation flow and decision-making history in chronological order.
Add new numbered sections as the spec progresses through direction changes or additional discussions.

---

## 1. Origin

사용자는 `techton-back`, `techton-front`, future services의 credential을 OCI Vault로 관리하고 싶다고 설명했고, 이를 위해서는 `oci-api`가 장기 API key signer 외에 Instance Principal 기반 인증도 지원해야 한다고 정리했다. 또한 `oci-api` 쪽 구현은 ecosystem parent spec의 sub spec으로 두고, 완료 후 `techton` 쪽 적용 phase로 넘어가야 한다고 결정했다.

## 2. Analysis

- 현재 `oci-api`는 `Email Delivery`와 `Object Storage`를 지원하지만, 인증은 `user_id + tenancy_id + fingerprint + private_key` 기반 signer로 고정되어 있다.
- crate는 OCI 공식 Rust SDK를 사용하는 것이 아니라, OCI REST API를 감싼 custom Rust client이므로 Vault/Keys와 Instance Principal도 같은 패턴으로 확장하는 것이 자연스럽다.
- consumer 프로젝트에서 Vault를 안전하게 쓰려면 장기 signing private key를 앱 환경변수에 두지 않는 방향이 필요하고, 그 핵심이 Instance Principal auth layer다.

## 3. Decision

- 기존 API key path는 유지하되, auth/provider abstraction을 도입해 Instance Principal을 추가한다.
- 기존 Email Delivery / Object Storage는 공통 auth layer 아래로 이관하고, consumer-facing API는 최대한 유지한다.
- 이 spec에는 Vault/Secrets, Keys, rotation primitive까지 포함하되, consumer rollout은 parent spec Phase 2에서 수행한다.

## 4. Refinement

- 사용자는 실제 Instance Principal 검증은 OCI instance에서만 가능하므로, OCI/Coolify 환경을 통한 test resource 생성 및 smoke/integration test도 필요하다고 지적했다.
- 이에 따라 sub spec에는 OCI-hosted smoke validation, test resource naming, cleanup 절차를 추가했다.

## 5. Refinement

- 사용자는 test용 resource container 추가/조작에 Coolify MCP를 쓰는 경계가 맞는지 물었고, 이에 대해 컨테이너 orchestration과 실제 OCI resource 조작을 분리해야 한다는 점을 정리했다.
- sub spec에는 Coolify MCP와 `oci-api`/OCI API 계층의 책임 경계를 명시적으로 추가했다.

## 6. Decision Lock

- 사용자는 남은 세부 결정은 내가 추천하는 기본값으로 잠그라고 요청했다.
- 이에 따라 `OCI_AUTH_MODE=api_key|instance_principal`, provider가 인증 헤더/상태를 생성하는 구조, phase-1 Vault/Keys endpoint 범위, test resource naming, cleanup 정책을 구현 기본값으로 고정했다.

<!-- Append numbered sections as the spec evolves -->
