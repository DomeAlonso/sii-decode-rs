import { useCallback, useEffect, useRef, useState } from "react";
import type { ChangeEvent, ReactNode } from "react";
import "./App.css";
import DecodeWorker from "./decode.worker?worker";
import type { DecodeResponse } from "./decode.worker";

type Tab = "decode" | "tracker";

type TrackerAnalysis = {
  analytics: {
    delivery_count: number;
    total_distance_km: number;
    total_revenue: number;
    unique_cargos: string[];
    unique_companies: string[];
    job_type_breakdown: Record<string, number>;
    brand_distance_km: Record<string, number>;
    cargo_category_coverage: Record<string, number>;
  };
  achievement_groups?: AchievementGroupInfo[];
  achievements: Array<{
    id: string;
    group?: string;
    steam_api_name?: string;
    display_name: string;
    description: string;
    status: "complete" | "in_progress" | "not_tracked";
    progress: {
      current: number;
      target: number;
      unit: string;
    };
    evidence: Array<{
      label: string;
      value: string;
      items?: string[];
      complete: boolean;
    }>;
  }>;
};

type AchievementGroupInfo = {
  id: string;
  name: string;
  steam_group_id?: number;
  steam_dlc_app_id?: number | null;
};

type Achievement = TrackerAnalysis["achievements"][number];

type AchievementGroup = AchievementGroupInfo & {
  achievements: Achievement[];
};

type AchievementStatus = TrackerAnalysis["achievements"][number]["status"];

type AchievementEvidence =
  TrackerAnalysis["achievements"][number]["evidence"][number];

function App() {
  const [file, setFile] = useState<File | null>(null);
  const [trackerFile, setTrackerFile] = useState<File | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>(() =>
    window.location.hash === "#tracker" ? "tracker" : "decode",
  );
  const [trackerState, setTrackerState] = useState<{
    status: "idle" | "loading" | "success" | "error";
    analysis: TrackerAnalysis | null;
    error: string | null;
  }>({ status: "idle", analysis: null, error: null });
  const textAreaRef = useRef<HTMLTextAreaElement>(null);
  const downloadRef = useRef<HTMLAnchorElement>(null);
  const workerRef = useRef<Worker | null>(null);

  useEffect(() => {
    workerRef.current = new DecodeWorker();
    return () => {
      workerRef.current?.terminate();
    };
  }, []);

  useEffect(() => {
    const handleHashChange = () => {
      setActiveTab(window.location.hash === "#tracker" ? "tracker" : "decode");
    };
    window.addEventListener("hashchange", handleHashChange);
    return () => {
      window.removeEventListener("hashchange", handleHashChange);
    };
  }, []);

  const selectTab = useCallback((tab: Tab) => {
    setActiveTab(tab);
    window.location.hash = tab === "tracker" ? "tracker" : "decode";
  }, []);

  const handleFile = useCallback((event: ChangeEvent<HTMLInputElement>) => {
    if (!event.target.files || event.target.files.length === 0) {
      return;
    }
    const selectedFile = event.target.files[0];
    if (textAreaRef.current) {
      textAreaRef.current.value = "Decoding...";
    }
    if (downloadRef.current) {
      if (downloadRef.current.href !== "#") {
        URL.revokeObjectURL(downloadRef.current.href);
      }
      downloadRef.current.href = "#";
    }
    setFile(selectedFile);
  }, []);

  useEffect(() => {
    if (!file || !workerRef.current) {
      return;
    }

    const worker = workerRef.current;

    const handleMessage = (event: MessageEvent<DecodeResponse>) => {
      if (event.data.type === "success") {
        const { result, blobUrl } = event.data;
        if (textAreaRef.current) {
          const PREVIEW_LIMIT = 100_000;
          if (result.length > PREVIEW_LIMIT) {
            textAreaRef.current.value =
              result.slice(0, PREVIEW_LIMIT) +
              `\n\n... (${(result.length / 1024 / 1024).toFixed(2)} MB total - download for full content)`;
          } else {
            textAreaRef.current.value = result;
          }
        }
        if (downloadRef.current) {
          downloadRef.current.href = blobUrl;
          downloadRef.current.download = file.name.replace(
            ".sii",
            "-decoded.sii",
          );
        }
      } else if (event.data.type === "decode-error") {
        if (textAreaRef.current) {
          textAreaRef.current.value = `Error: ${event.data.message}`;
        }
      }
    };

    worker.addEventListener("message", handleMessage);

    const reader = new FileReader();
    reader.onload = (e) => {
      const arrayBuffer = e.target?.result as ArrayBuffer;
      worker.postMessage({ type: "decode", buffer: arrayBuffer }, [
        arrayBuffer,
      ]);
    };
    reader.readAsArrayBuffer(file);

    return () => {
      worker.removeEventListener("message", handleMessage);
    };
  }, [file]);

  const handleTrackerFile = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      if (!event.target.files || event.target.files.length === 0) {
        return;
      }
      setTrackerState({ status: "loading", analysis: null, error: null });
      setTrackerFile(event.target.files[0]);
    },
    [],
  );

  useEffect(() => {
    if (!trackerFile || !workerRef.current) {
      return;
    }

    const worker = workerRef.current;

    const handleMessage = (event: MessageEvent<DecodeResponse>) => {
      if (event.data.type === "analysis-success") {
        try {
          setTrackerState({
            status: "success",
            analysis: JSON.parse(event.data.result) as TrackerAnalysis,
            error: null,
          });
        } catch {
          setTrackerState({
            status: "error",
            analysis: null,
            error: "Tracker output could not be read.",
          });
        }
      } else if (event.data.type === "analysis-error") {
        setTrackerState({
          status: "error",
          analysis: null,
          error: event.data.message,
        });
      }
    };

    worker.addEventListener("message", handleMessage);

    const reader = new FileReader();
    reader.onload = (e) => {
      const arrayBuffer = e.target?.result as ArrayBuffer;
      worker.postMessage({ type: "analyze", buffer: arrayBuffer }, [
        arrayBuffer,
      ]);
    };
    reader.readAsArrayBuffer(trackerFile);

    return () => {
      worker.removeEventListener("message", handleMessage);
    };
  }, [trackerFile]);

  return (
    <main className="app-shell">
      <header className="app-header">
        <h1>SII Decode</h1>
        <nav className="tabs" aria-label="Primary">
          <button
            type="button"
            className={activeTab === "decode" ? "active" : ""}
            onClick={() => selectTab("decode")}
          >
            Decode
          </button>
          <button
            type="button"
            className={activeTab === "tracker" ? "active" : ""}
            onClick={() => selectTab("tracker")}
          >
            ETS2 Tracker
          </button>
        </nav>
      </header>

      {activeTab === "decode" ? (
        <section className="panel" aria-labelledby="decode-title">
          <h2 id="decode-title">Decode</h2>
          <div className="file-row">
            <input
              type="file"
              id="file"
              data-testid="file-upload"
              onChange={handleFile}
            />
          </div>
          <textarea
            id="output"
            rows={20}
            ref={textAreaRef}
            data-testid="file-display"
            spellCheck="false"
            readOnly
          />
          <div>
            <a href="#" ref={downloadRef} data-testid="file-download">
              Download decoded file
            </a>
          </div>
        </section>
      ) : (
        <TrackerView state={trackerState} onFileChange={handleTrackerFile} />
      )}

      <p className="footer">
        Your file is not uploaded to any server, it is decoded using your own
        browser.
        <br />
        This tools is{" "}
        <a href="https://github.com/fangyi-zhou/sii-decode-rs/">open source</a>.
        If you encounter any issues, please report them{" "}
        <a href="https://github.com/fangyi-zhou/sii-decode-rs/issues">
          on GitHub
        </a>
        .
      </p>
    </main>
  );
}

function TrackerView({
  state,
  onFileChange,
}: {
  state: {
    status: "idle" | "loading" | "success" | "error";
    analysis: TrackerAnalysis | null;
    error: string | null;
  };
  onFileChange: (event: ChangeEvent<HTMLInputElement>) => void;
}) {
  return (
    <section className="panel" aria-labelledby="tracker-title">
      <h2 id="tracker-title">ETS2 Tracker</h2>
      <div className="file-row">
        <input
          type="file"
          id="tracker-file"
          data-testid="tracker-file-upload"
          onChange={onFileChange}
        />
      </div>

      {state.status === "loading" ? (
        <p data-testid="tracker-status">Analyzing save...</p>
      ) : null}

      {state.status === "error" ? (
        <p className="error" data-testid="tracker-error">
          Error: {state.error}
        </p>
      ) : null}

      {state.status === "success" && state.analysis ? (
        <div className="tracker-results" data-testid="tracker-results">
          <AnalyticsSummary analysis={state.analysis} />
          <AchievementGroups analysis={state.analysis} />
        </div>
      ) : null}
    </section>
  );
}

function AnalyticsSummary({ analysis }: { analysis: TrackerAnalysis }) {
  const analytics = analysis.analytics;
  return (
    <section className="summary-grid" aria-label="Analytics summary">
      <Metric label="Deliveries" value={analytics.delivery_count} />
      <Metric label="Distance" value={`${analytics.total_distance_km} km`} />
      <Metric label="Revenue" value={formatCurrency(analytics.total_revenue)} />
      <Metric label="Cargos" value={analytics.unique_cargos.length} />
      <Metric label="Companies" value={analytics.unique_companies.length} />
      <Metric
        label="Job Types"
        value={Object.keys(analytics.job_type_breakdown).length}
      />
      <Metric
        label="Achievements"
        value={`${
          analysis.achievements.filter(
            (achievement) => achievement.status === "complete",
          ).length
        } / ${analysis.achievements.length}`}
      />
    </section>
  );
}

function Metric({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

const DEFAULT_ACHIEVEMENT_GROUP: AchievementGroupInfo = {
  id: "base_game",
  name: "Base Game",
};

function groupAchievements(analysis: TrackerAnalysis): AchievementGroup[] {
  const groupInfos = analysis.achievement_groups?.length
    ? analysis.achievement_groups
    : [DEFAULT_ACHIEVEMENT_GROUP];
  const groups = new Map<string, AchievementGroup>(
    groupInfos.map((group) => [group.id, { ...group, achievements: [] }]),
  );
  for (const achievement of analysis.achievements) {
    const groupId = achievement.group ?? groupInfos[0].id;
    let group = groups.get(groupId);
    if (!group) {
      group = { id: groupId, name: groupId, achievements: [] };
      groups.set(groupId, group);
    }
    group.achievements.push(achievement);
  }
  return [...groups.values()].filter((group) => group.achievements.length > 0);
}

function AchievementGroups({ analysis }: { analysis: TrackerAnalysis }) {
  const groups = groupAchievements(analysis);
  return (
    <section className="achievement-groups" aria-label="Achievements">
      {groups.map((group, index) => (
        <AchievementGroupSection
          key={group.id}
          group={group}
          defaultExpanded={index === 0}
        />
      ))}
    </section>
  );
}

function AchievementGroupSection({
  group,
  defaultExpanded,
}: {
  group: AchievementGroup;
  defaultExpanded: boolean;
}) {
  const [expanded, setExpanded] = useState(defaultExpanded);
  const contentId = `achievement-group-${group.id}`;
  const total = group.achievements.length;
  const complete = group.achievements.filter(
    (achievement) => achievement.status === "complete",
  ).length;
  const tracked = group.achievements.filter(
    (achievement) => achievement.status !== "not_tracked",
  ).length;

  return (
    <section className="achievement-group" data-testid="achievement-group">
      <h3 className="achievement-group-heading">
        <button
          type="button"
          aria-expanded={expanded}
          aria-controls={contentId}
          onClick={() => setExpanded((value) => !value)}
        >
          <span className="achievement-group-chevron" aria-hidden="true" />
          <span className="achievement-group-name">{group.name}</span>
          <span className="achievement-group-summary">
            {complete} / {total} complete
            {tracked < total ? ` · ${tracked} tracked` : ""}
          </span>
        </button>
      </h3>
      {expanded ? (
        <div id={contentId} className="achievement-group-content">
          <p className="achievement-group-count">Achievements ({total})</p>
          <AchievementList
            achievements={group.achievements}
            label={`${group.name} achievements`}
          />
        </div>
      ) : null}
    </section>
  );
}

function AchievementList({
  achievements,
  label,
}: {
  achievements: Achievement[];
  label: string;
}) {
  return (
    <section className="achievements" aria-label={label}>
      {achievements.map((achievement) => (
        <article
          className={`achievement ${achievement.status}`}
          key={achievement.id}
        >
          <div className="achievement-heading">
            <div>
              <h4>{achievement.display_name}</h4>
              <p>{achievement.description}</p>
            </div>
            <span className={`status ${achievement.status}`}>
              {statusLabel(achievement.status)}
            </span>
          </div>
          {achievement.status === "not_tracked" ? null : (
            <>
              <progress
                value={achievement.progress.current}
                max={achievement.progress.target}
                aria-label={`${achievement.display_name} progress`}
              />
              <div className="progress-text">
                {achievement.progress.current} / {achievement.progress.target}{" "}
                {achievement.progress.unit}
              </div>
            </>
          )}
          <AchievementEvidenceList
            evidence={achievement.evidence}
            itemLabel={evidenceListItemLabel(achievement.id)}
          />
        </article>
      ))}
    </section>
  );
}

function AchievementEvidenceList({
  evidence,
  itemLabel,
}: {
  evidence: AchievementEvidence[];
  itemLabel: string | null;
}) {
  const [expandedEvidence, setExpandedEvidence] =
    useState<AchievementEvidence | null>(null);
  const expandedItems = expandedEvidence?.items ?? [];

  return (
    <>
      <ul className="evidence-list">
        {evidence.map((item) => {
          const items = item.items ?? [];
          const hasList = itemLabel !== null && items.length > 1;
          return (
            <li key={item.label}>
              <span className="evidence-label">{item.label}</span>
              {hasList ? (
                <button
                  type="button"
                  className="evidence-summary"
                  onClick={() => setExpandedEvidence(item)}
                >
                  View {items.length} {itemLabel}
                </button>
              ) : (
                <span className="evidence-value">{item.value}</span>
              )}
            </li>
          );
        })}
      </ul>

      {expandedEvidence ? (
        <div
          className="evidence-popout-backdrop"
          onClick={() => setExpandedEvidence(null)}
        >
          <section
            className="evidence-popout"
            role="dialog"
            aria-modal="true"
            aria-labelledby="evidence-popout-title"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="evidence-popout-heading">
              <div>
                <h4 id="evidence-popout-title">{expandedEvidence.label}</h4>
                <p>
                  {expandedItems.length} {itemLabel ?? "items"}
                </p>
              </div>
              <button
                type="button"
                className="evidence-popout-close"
                onClick={() => setExpandedEvidence(null)}
              >
                Close
              </button>
            </div>
            <ul className="cargo-popout-list">
              {expandedItems.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </section>
        </div>
      ) : null}
    </>
  );
}

function statusLabel(status: AchievementStatus) {
  switch (status) {
    case "complete":
      return "Complete";
    case "in_progress":
      return "In progress";
    case "not_tracked":
      return "Not tracked";
  }
}

const EVIDENCE_ITEM_LABELS: Record<string, string> = {
  reliable_contractor: "companies",
  experience_beats_all: "cargos",
  all_is_possible: "cargos",
  size_matters: "cargos",
  the_bigger_the_better: "cargos",
  giant: "cargos",
  big_brother: "cargos",
  not_a_canoe: "cargos",
  volvo_trucks_lover: "deliveries",
  scania_trucks_lover: "deliveries",
  turkish_delight: "deliveries",
};

function evidenceListItemLabel(achievementId: string) {
  return EVIDENCE_ITEM_LABELS[achievementId] ?? null;
}

function formatCurrency(value: number) {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "EUR",
    maximumFractionDigits: 0,
  }).format(value);
}

export default App;
