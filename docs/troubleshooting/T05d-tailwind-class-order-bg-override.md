# Panel의 innerClassName으로 준 배경이 기본 배경에 덮임
- task: T05d · PR: (PR 생성 후 번호) · 날짜: 2026-11-10
- 영역: frontend/components/ui

## 증상
`<Panel innerClassName="bg-bg ...">`처럼 배경을 바꿔도 기본 `bg-surface`가 그대로 보인다.

## 원인
Tailwind는 클래스 문자열의 순서가 아니라 생성된 CSS의 순서가 이긴다. 같은 속성의 두 유틸리티를 함께 주면 어느 쪽이 이길지 클래스를 쓴 순서와 무관하다.

## 해결
Panel이 innerClassName에 `bg-`로 시작하는 클래스가 있으면 기본 `bg-surface`를 뺀다.

## 재발 방지
공통 컴포넌트가 기본 스타일을 깔고 className으로 덮어쓰게 할 때는 덮어쓸 속성을 조건부로 뺀다(예: Panel의 배경). 같은 이유로 LiveCard가 Avatar에 주는 `size-6`은 Avatar `size-*`에 이기지 못한다(T06c에서 확인).
