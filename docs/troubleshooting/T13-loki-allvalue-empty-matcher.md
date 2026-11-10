# Grafana 로그 대시보드에서 변수 All 이 Loki 쿼리 오류를 낸다
- task: T13 · PR: #62 · 날짜: 2026-11-10
- 영역: deploy/monitoring (Grafana·Loki·node-exporter)

## 증상
"로그 검색" 대시보드의 두 패널이 빨간 경고만 표시하고 데이터가 없다(서비스 변수 기본값 All).

## 원인
변수 All의 `allValue`를 `.*`로 두어 쿼리가 `{service=~".*"}`가 됐다. Loki는 빈 문자열과 매칭되는 셀렉터만 있는 쿼리를 거절한다(비어 있지 않은 매처가 최소 하나 필요).

## 해결
서비스 변수 `allValue`를 `.+`로 바꿨다. 레벨 변수는 두 번째 매처라 `.*`여도 되고, 이 경우 level 라벨이 없는 줄(JSON 파싱 못 한 로그)도 같이 나온다.

## 재발 방지
- 같은 확인 중 알게 된 것: Docker Desktop(WSL)에서는 node-exporter의 `/:/host:ro,rslave` 마운트가 "not a shared mount"로 실패해서 `rslave`를 뺐고, 이 환경에는 `/` 마운트포인트 시리즈가 없어 디스크 패널은 로컬에서 비어 있다(실제 Linux 서버에서는 나온다).
- Loki 3.x는 `service_name` 라벨을 자동으로 붙이므로 `limits_config.discover_service_name: []`로 끈다(라벨은 service·level만).
