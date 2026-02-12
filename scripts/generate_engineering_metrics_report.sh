#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

OUTPUT_DIR="${1:-${REPO_ROOT}/docs/ops-reports}"
WINDOW_DAYS="${METRICS_WINDOW_DAYS:-7}"
REPOSITORY="${GITHUB_REPOSITORY:-}"

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI is required to generate engineering metrics." >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to generate engineering metrics." >&2
  exit 1
fi

if [[ -z "${REPOSITORY}" ]]; then
  REPOSITORY="$(gh repo view --json nameWithOwner --jq '.nameWithOwner')"
fi

date_minus_days_utc() {
  local days="$1"
  if date -u -d "-${days} days" +"%Y-%m-%dT%H:%M:%SZ" >/dev/null 2>&1; then
    date -u -d "-${days} days" +"%Y-%m-%dT%H:%M:%SZ"
  else
    date -u -v-"${days}"d +"%Y-%m-%dT%H:%M:%SZ"
  fi
}

NOW_UTC="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
SINCE_UTC="$(date_minus_days_utc "${WINDOW_DAYS}")"
SINCE_DATE="${SINCE_UTC%%T*}"

mkdir -p "${OUTPUT_DIR}"

JSON_OUT="${OUTPUT_DIR}/engineering-metrics.json"
MD_OUT="${OUTPUT_DIR}/engineering-metrics.md"

echo "Collecting merged PRs since ${SINCE_DATE} from ${REPOSITORY}..."
MERGED_PRS_RAW="$(gh pr list \
  --repo "${REPOSITORY}" \
  --state merged \
  --search "merged:>=${SINCE_DATE}" \
  --json number,title,url,createdAt,mergedAt,mergeCommit \
  --limit 200)"
MERGED_PRS="$(jq --arg since "${SINCE_UTC}" '[.[] | select(.mergedAt >= $since)]' <<< "${MERGED_PRS_RAW}")"

echo "Collecting CI runs on main..."
MAIN_CI_RUNS_RAW="$(gh run list \
  --repo "${REPOSITORY}" \
  --workflow "CI" \
  --branch main \
  --event push \
  --json headSha,conclusion,createdAt,updatedAt,url,displayTitle \
  --limit 250)"
MAIN_CI_RUNS="$(jq --arg since "${SINCE_UTC}" '[.[] | select(.createdAt >= $since) | select(.conclusion != null)]' <<< "${MAIN_CI_RUNS_RAW}")"

echo "Collecting PR CI runs for flaky-rate proxy..."
PR_CI_RUNS_RAW="$(gh run list \
  --repo "${REPOSITORY}" \
  --workflow "CI" \
  --event pull_request \
  --json headSha,conclusion,createdAt,url,displayTitle \
  --limit 400)"
PR_CI_RUNS="$(jq --arg since "${SINCE_UTC}" '[.[] | select(.createdAt >= $since) | select(.conclusion != null) | select(.headSha != null)]' <<< "${PR_CI_RUNS_RAW}")"

METRICS_JSON="$(jq -n \
  --arg repository "${REPOSITORY}" \
  --arg now "${NOW_UTC}" \
  --arg since "${SINCE_UTC}" \
  --argjson window_days "${WINDOW_DAYS}" \
  --argjson prs "${MERGED_PRS}" \
  --argjson main_runs "${MAIN_CI_RUNS}" \
  --argjson pr_runs "${PR_CI_RUNS}" '
  def median($arr):
    ($arr | sort) as $s
    | ($s | length) as $n
    | if $n == 0 then null
      elif ($n % 2) == 1 then $s[($n / 2 | floor)]
      else (($s[($n / 2 | floor) - 1] + $s[($n / 2 | floor)]) / 2)
      end;
  def percentile($arr; $p):
    ($arr | sort) as $s
    | ($s | length) as $n
    | if $n == 0 then null
      else $s[((($n - 1) * $p) | floor)]
      end;
  def hours_between($a; $b):
    (($b | fromdateiso8601) - ($a | fromdateiso8601)) / 3600;

  ($prs | map({
    number,
    title,
    url,
    merge_sha: .mergeCommit.oid,
    created_at: .createdAt,
    merged_at: .mergedAt,
    lead_time_hours: hours_between(.createdAt; .mergedAt)
  })) as $changes
  | ($changes | map(.lead_time_hours)) as $lead_time_hours
  | ($main_runs | sort_by(.createdAt)) as $ordered_main_runs
  | ($ordered_main_runs | map(select(.conclusion == "failure"))) as $main_failures
  | ($changes | map(. as $change | select(($main_failures | map(.headSha) | index($change.merge_sha)) != null))) as $failed_changes
  | (
      $ordered_main_runs
      | map(select(.conclusion == "failure") as $f
        | (
            $ordered_main_runs
            | map(select(.conclusion == "success" and (.createdAt > $f.createdAt)))
            | first
          ) as $recovery
        | select($recovery != null)
        | {
            failed_run_created_at: $f.createdAt,
            recovered_at: $recovery.createdAt,
            mttr_hours: hours_between($f.createdAt; $recovery.createdAt)
          }
      )
    ) as $recoveries
  | (
      $pr_runs
      | group_by(.headSha)
      | map({
          head_sha: .[0].headSha,
          conclusions: (map(.conclusion) | unique)
        })
    ) as $pr_run_groups
  | (
      $pr_run_groups
      | map(select(
          (.conclusions | index("success")) != null
          and (
            (.conclusions | index("failure")) != null
            or (.conclusions | index("cancelled")) != null
            or (.conclusions | index("timed_out")) != null
          )
        ))
    ) as $flaky_groups
  | {
      repository: $repository,
      generated_at: $now,
      window_days: $window_days,
      since_utc: $since,
      merged_pr_count: ($changes | length),
      lead_time_median_hours: median($lead_time_hours),
      lead_time_p90_hours: percentile($lead_time_hours; 0.9),
      lead_time_sample_count: ($lead_time_hours | length),
      failed_change_count: ($failed_changes | length),
      change_failure_rate_pct: (
        if ($changes | length) == 0
        then null
        else (($failed_changes | length) / ($changes | length) * 100)
        end
      ),
      mttr_event_count: ($recoveries | length),
      mttr_mean_hours: (
        if ($recoveries | length) == 0
        then null
        else (($recoveries | map(.mttr_hours) | add) / ($recoveries | length))
        end
      ),
      flaky_sha_count: ($flaky_groups | length),
      flaky_population_count: ($pr_run_groups | length),
      flaky_rate_pct: (
        if ($pr_run_groups | length) == 0
        then null
        else (($flaky_groups | length) / ($pr_run_groups | length) * 100)
        end
      ),
      merged_changes: $changes,
      failed_changes: $failed_changes,
      mttr_recoveries: $recoveries,
      flaky_groups: $flaky_groups,
      main_ci_runs_considered: ($ordered_main_runs | length),
      pr_ci_runs_considered: ($pr_runs | length)
    }
')"

printf '%s\n' "${METRICS_JSON}" > "${JSON_OUT}"

format_number() {
  local value="$1"
  if [[ -z "${value}" || "${value}" == "null" ]]; then
    echo "n/a"
    return
  fi
  printf "%.2f" "${value}"
}

lead_median="$(format_number "$(jq -r '.lead_time_median_hours' <<< "${METRICS_JSON}")")"
lead_p90="$(format_number "$(jq -r '.lead_time_p90_hours' <<< "${METRICS_JSON}")")"
cfr="$(format_number "$(jq -r '.change_failure_rate_pct' <<< "${METRICS_JSON}")")"
mttr="$(format_number "$(jq -r '.mttr_mean_hours' <<< "${METRICS_JSON}")")"
flaky="$(format_number "$(jq -r '.flaky_rate_pct' <<< "${METRICS_JSON}")")"

cat > "${MD_OUT}" <<EOF
# Engineering Metrics Report

- Repository: \`${REPOSITORY}\`
- Generated (UTC): \`${NOW_UTC}\`
- Window: last \`${WINDOW_DAYS}\` days (\`${SINCE_UTC}\` -> \`${NOW_UTC}\`)

## Snapshot

| Metric | Value | Notes |
| --- | --- | --- |
| Lead time for change (median) | ${lead_median}h | Merged PR create->merge duration |
| Lead time for change (p90) | ${lead_p90}h | Tail latency for merged PRs |
| Change failure rate (proxy) | ${cfr}% | Merged PRs whose merge SHA had failing \`CI\` on \`main\` |
| Mean time to recovery (proxy) | ${mttr}h | Failure->next successful \`CI\` run on \`main\` |
| Flaky test rate (proxy) | ${flaky}% | PR SHAs with both fail/cancel + success in CI |

## Sample Sizes

- Merged PRs considered: $(jq -r '.merged_pr_count' <<< "${METRICS_JSON}")
- Main CI runs considered: $(jq -r '.main_ci_runs_considered' <<< "${METRICS_JSON}")
- PR CI runs considered: $(jq -r '.pr_ci_runs_considered' <<< "${METRICS_JSON}")
- MTTR recovery events: $(jq -r '.mttr_event_count' <<< "${METRICS_JSON}")
- Flaky SHA population: $(jq -r '.flaky_population_count' <<< "${METRICS_JSON}")

## Recently Merged Changes
EOF

jq -r '
  if (.merged_changes | length) == 0 then
    "- No merged PRs in the selected window."
  else
    (.merged_changes
      | sort_by(.merged_at)
      | reverse
      | .[0:12]
      | .[]
      | "- #\(.number) \(.title) (lead: \((((.lead_time_hours * 100) | round) / 100))h) - \(.url)")
  end
' <<< "${METRICS_JSON}" >> "${MD_OUT}"

cat >> "${MD_OUT}" <<EOF

## Metric Definitions

- **Lead time for change**: time from PR creation to PR merge.
- **Change failure rate (proxy)**: fraction of merged PRs whose merge commit triggered a failed \`CI\` run on \`main\`.
- **MTTR (proxy)**: average time between a failed \`CI\` run on \`main\` and the next successful \`CI\` run.
- **Flaky test rate (proxy)**: fraction of PR head SHAs that saw both success and failure/cancel/timed_out CI conclusions in the window.

## Triage Notes

- Investigate any increase in change failure rate or MTTR before increasing merge throughput.
- Track persistent flaky SHAs/checks and either stabilize tests or quarantine unstable coverage.
- Use this report as an input to weekly dependency, CI-regression, and security triage.
EOF

echo "Generated metrics report:"
echo "  - ${MD_OUT}"
echo "  - ${JSON_OUT}"
