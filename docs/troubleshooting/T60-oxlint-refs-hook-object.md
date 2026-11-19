# oxlint `react(refs)`가 ref를 담은 훅 반환 객체의 모든 속성 접근을 경고
- task: T60 · PR: (이 PR) · 날짜: 2026-11-19
- 영역: frontend/studio

## 증상
`useLayoutEditor()`가 `{ layout, selected, boxRef, bind(key) }`를 돌려주고 컴포넌트가 `e.layout`·`e.bind(key)`를 렌더 중에 쓰자 `oxlint --deny-warnings`가 `Cannot access refs during render`(4건)로 실패했다. 실제로 렌더 중 `ref.current`를 읽지는 않는다.

## 원인
React Compiler 계열 규칙이 ref(`useRef` 결과)를 담은 객체를 통째로 ref로 보고, 그 객체의 다른 속성 접근·그 객체의 함수 호출(ref를 닫아 둔 `bind(key)`)도 "렌더 중 ref 읽기"로 판단한다.

## 해결
- 렌더 값(`layout`·`selected`)은 부모가 따로 prop으로 넘긴다.
- 컴포넌트에서 ref·핸들러를 구조 분해해 쓴다: `({ editor: { boxRef, handlers } })` → `ref={boxRef}`, `{...handlers}`.
- 요소별 핸들러를 만드는 함수(`bind(key)`) 대신 고정 핸들러 객체 + `data-key` 속성으로 요소를 구분한다.

## 재발 방지
훅이 ref와 렌더 값을 함께 돌려주면, 화면에서는 ref·핸들러를 구조 분해하고 렌더 값은 별도 prop으로 받는다(`usePlayer`의 `const { videoRef, … } = p`와 같은 방식). 렌더 중 "핸들러를 만드는 함수"를 부르지 않는다.
