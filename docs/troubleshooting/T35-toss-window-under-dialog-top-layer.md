# 토스 결제창이 모달 <dialog> 뒤에 깔려 결제 불가
- task: T35 · PR: #124 · 날짜: 2026-11-11
- 영역: frontend/wallet, frontend/components/ui/Modal

## 증상
충전·후원 모달의 결제위젯에서 "결제하기"를 누르면 토스 결제창이 열리지만 우리 모달 뒤에 깔려 클릭이 안 된다. 헤드리스에서 화면 가운데 `elementFromPoint`가 모달 안 위젯 iframe을 가리켰다.

## 원인
`Modal`은 `<dialog>.showModal()`이라 브라우저 top layer에 올라가고 나머지 문서는 inert가 된다. 토스 결제위젯 v2 `requestPayment`(PC)는 `body`에 `#__tosspayments_payment-gateway_dimmer__`(fixed, z-index 9999999)를 붙인다. z-index로는 top layer를 넘을 수 없고, inert라 클릭도 안 된다. 실브라우저도 규격상 같다.

## 해결
- `useTossPayment.pay()`가 결제창을 띄우기 전에 위젯이 들어 있는 dialog를 `close()` → `show()`(비모달)로 내리고, `requestPayment`가 끝나면(취소·실패) `close()` → `showModal()`로 되돌린다. 위젯 DOM은 그대로 남는다.
- `close()`가 보내는 `close` 이벤트로 창이 닫히지 않게 `Modal`의 `onClose`는 `!dialog.open`일 때만 부른다(다시 열린 경우는 무시).
- 비모달 상태에서도 같은 자리에 있도록 dialog에 `fixed inset-0`.

## 재발 방지
- 외부 SDK가 `body`에 오버레이를 붙이는 기능(결제창·인증창)은 `showModal()` 모달 안에서 부르면 가려진다. 부르기 전에 top layer에서 내린다.
- 확인: 결제창을 띄운 뒤 `document.elementFromPoint(innerWidth/2, innerHeight/2).closest('#__tosspayments_payment-gateway_dimmer__')`가 있어야 한다.
