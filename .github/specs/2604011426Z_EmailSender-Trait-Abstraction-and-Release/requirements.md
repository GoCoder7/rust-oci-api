# Requirements

## Background

`oci-api` crate (v0.5.0)의 `EmailDelivery`는 concrete struct로만 구현되어 있어, 의존 프로젝트에서 테스트 시 이메일 전송을 교체하거나 mock할 방법이 없다. 현재 `techton-back`은 `#[cfg(test)]` 컴파일 타임 분기로 우회하고 있으나, 이 방식은 프로덕션 코드 경로와 테스트 경로가 달라서 실제 호출 검증이 불가능하다.

## User Stories

### Story 1: trait 기반 이메일 전송 추상화

개발자로서, `EmailDelivery`를 trait으로 추상화하여 테스트에서 mock 구현체를 주입할 수 있게 하고 싶다. 이를 통해 실제 OCI API를 호출하지 않고도 이메일 전송 로직을 검증할 수 있어야 한다.

### Story 2: crate 문서 및 릴리스

crate 사용자로서, 새로운 trait과 사용 예시가 README와 doc comment에 반영된 새 버전을 `crates.io`에서 받을 수 있어야 한다.

### Story 3: release skill 파이프라인

`oci-api` 프로젝트에 agent/skill 기반 릴리스 파이프라인을 갖추어, 버전 bump → changelog → publish 과정을 체계적으로 관리할 수 있어야 한다.

## Acceptance Criteria

- [ ] `EmailSender` trait이 `send`, `get_email_configuration`, `list_senders` 메서드를 정의한다
- [ ] `EmailDelivery`가 `EmailSender`를 impl한다
- [ ] 기존 public API는 breaking change 없이 유지된다
- [ ] trait과 mock 예시가 doc comment와 README에 포함된다
- [ ] 기존 단위 테스트 18개+ 통과
- [ ] 기존 integration test (`#[ignore]`) 통과
- [ ] crate 버전이 minor bump된다 (0.5.0 → 0.6.0)
- [ ] `crates.io`에 새 버전이 publish된다
- [ ] release 관련 skill 정의가 `.github/skills/`에 존재한다

## Scope

### Included

- `EmailSender` trait 정의 및 `EmailDelivery` impl
- `MockEmailSender` 예시 (doc comment 또는 `examples/`)
- README 업데이트 (trait 사용법, mock 패턴)
- CHANGELOG 업데이트
- crate version bump 및 crates.io publish
- release skill 정의 (`.github/skills/release/`)

### Excluded

- OCI Logging 조회 기능 (별도 spec으로 분리)
- 다른 OCI 서비스(Object Storage, Vault)의 trait 추상화
- `techton-back` 측 통합 (별도 spec: `OCI-Email-Test-Integration`)
