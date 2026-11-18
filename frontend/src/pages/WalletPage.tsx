import BeatAmount from "../components/ui/BeatAmount.tsx";
import Button from "../components/ui/Button.tsx";
import { ChoiceChip } from "../components/ui/Chip.tsx";
import EmptyState from "../components/ui/EmptyState.tsx";
import ErrorScreen from "../components/ui/ErrorScreen.tsx";
import Panel from "../components/ui/Panel.tsx";
import Skeleton from "../components/ui/Skeleton.tsx";
import ChargePanel from "../features/wallet/components/ChargePanel.tsx";
import LedgerList from "../features/wallet/components/LedgerList.tsx";
import { kstMonths } from "../features/wallet/format.ts";
import {
  LEDGER_FILTERS,
  useWallet,
} from "../features/wallet/hooks/useWallet.ts";
import { errorKindOf } from "../hooks/errorKind.ts";

const MONTHS = kstMonths();

// 내 지갑: 잔액 + 내역(유형·월 필터) 옆에 충전 패널(좁은 폭에서는 아래로). 로직은 useWallet·useCharge.
export default function WalletPage() {
  const w = useWallet();
  const { query } = w;
  if (!w.me) return null;

  return (
    <div className="mx-auto flex max-w-[1200px] flex-wrap gap-7">
      <div className="flex min-w-0 flex-[999_1_560px] flex-col gap-5">
        <h1 className="text-[22px] font-black sm:text-[26px]">내 지갑</h1>

        <Panel
          tone="hero"
          innerClassName="flex flex-wrap items-center gap-4 p-[18px] sm:gap-5 sm:p-6"
        >
          <div className="flex min-w-0 grow flex-col gap-1">
            <span className="text-sm text-text-muted">보유 비트</span>
            <BeatAmount
              amount={w.balance}
              unit
              className="text-3xl sm:text-[40px]"
            />
            <span className="text-xs text-text-dim">
              1 Beat = 1원 · 보유 한도 3,000,000
            </span>
          </div>
          {/* 충전 패널은 같은 화면에 있다: 좁은 폭에서는 패널 위치로 이동 */}
          <Button
            size="lg"
            className="max-sm:w-full"
            onClick={() =>
              document
                .getElementById("charge-panel")
                ?.scrollIntoView({ behavior: "smooth", block: "start" })
            }
          >
            충전하기
          </Button>
        </Panel>

        <div className="flex flex-wrap items-center gap-2">
          <h2 className="mr-auto text-lg font-black max-sm:w-full">
            사용 내역
          </h2>
          {/* 좁은 폭: 칩과 월 선택을 한 줄 가로 스크롤로 (시안 보드는 칩이 화면 밖으로 밀려 안 보임) */}
          <div className="flex items-center gap-2 max-sm:w-full max-sm:overflow-x-auto sm:contents">
            <div
              role="group"
              aria-label="내역 유형"
              className="flex shrink-0 gap-2 sm:flex-wrap"
            >
              {LEDGER_FILTERS.map((f) => (
                <ChoiceChip
                  key={f.type}
                  selected={w.type === f.type}
                  onClick={() => w.setType(f.type)}
                >
                  {f.label}
                </ChoiceChip>
              ))}
            </div>
            <label className="sr-only" htmlFor="wallet-month">
              월
            </label>
            <select
              id="wallet-month"
              value={w.month}
              onChange={(e) => w.setMonth(e.target.value)}
              className="chamfer h-9 shrink-0 bg-surface-2 px-2.5 text-[13px] text-text [--c:6px]"
            >
              <option value="">전체 기간</option>
              {MONTHS.map((m) => (
                <option key={m.value} value={m.value}>
                  {m.label}
                </option>
              ))}
            </select>
          </div>
        </div>

        {query.isError ? (
          <ErrorScreen
            {...errorKindOf(query.error)}
            onRetry={() => void query.refetch()}
          />
        ) : query.isPending ? (
          <Skeleton className="h-64 w-full" card />
        ) : w.items.length === 0 ? (
          <EmptyState
            title="내역이 없어요"
            description="충전하거나 후원하면 여기에 기록돼요."
          />
        ) : (
          <>
            <LedgerList items={w.items} />
            {query.hasNextPage && (
              <div className="flex justify-center">
                <Button
                  variant="secondary"
                  onClick={() => void query.fetchNextPage()}
                  loading={query.isFetchingNextPage}
                >
                  더 보기
                </Button>
              </div>
            )}
          </>
        )}
        <p className="text-xs text-text-dim">
          기록은 지워지지 않아요. 유료 클러치는 방송별로 묶어서 보여줘요.
        </p>
      </div>

      <ChargePanel balance={w.balance} />
    </div>
  );
}
