/** `11월 9일 21:42` (KST). 화면 전체에서 날짜+시각 표기는 이것 하나로 맞춘다. */
export const kstStamp = (iso: string) =>
  new Date(iso).toLocaleString('ko-KR', { timeZone: 'Asia/Seoul', month: 'long', day: 'numeric', hour: '2-digit', minute: '2-digit', hour12: false })
