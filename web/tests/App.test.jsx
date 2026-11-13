import { test, expect, vi, beforeEach } from "vitest";
import App from "../src/App";
import "@testing-library/jest-dom/vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("../src/decode.worker?worker", () => {
  return {
    default: class MockWorker {
      onmessage = null;
      onerror = null;
      onmessageerror = null;
      messageListeners = new Set();

      postMessage(data) {
        setTimeout(() => {
          if (data.type === "decode") {
            const bytes = new Uint8Array(data.buffer);
            const text = new TextDecoder().decode(bytes);

            if (text.startsWith("SiiN")) {
              const blob = new Blob([text], { type: "text/plain" });
              const blobUrl = URL.createObjectURL(blob);
              this.dispatchMessage({
                data: { type: "success", result: text, blobUrl },
              });
            } else {
              this.dispatchMessage({
                data: {
                  type: "decode-error",
                  message: "Invalid SII file format",
                },
              });
            }
          } else if (data.type === "analyze") {
            const bytes = new Uint8Array(data.buffer);
            const text = new TextDecoder().decode(bytes);

            if (text === "BSII-old") {
              this.dispatchMessage({
                data: {
                  type: "analysis-success",
                  result: JSON.stringify({
                    analytics: {
                      delivery_count: 1,
                      total_distance_km: 100,
                      total_revenue: 1000,
                      unique_cargos: ["gravel"],
                      unique_companies: ["lkwlog"],
                      job_type_breakdown: { cargo: 1 },
                      brand_distance_km: {},
                      cargo_category_coverage: {},
                    },
                    achievements: [
                      {
                        id: "long_hauler",
                        display_name: "Long Hauler",
                        description:
                          "Complete a delivery that was greater than 2,000 km",
                        status: "in_progress",
                        progress: { current: 100, target: 2000, unit: "km" },
                        evidence: [],
                      },
                    ],
                  }),
                },
              });
            } else if (text.startsWith("BSII")) {
              this.dispatchMessage({
                data: {
                  type: "analysis-success",
                  result: JSON.stringify({
                    analytics: {
                      delivery_count: 2,
                      total_distance_km: 1844,
                      total_revenue: 21760.5,
                      unique_cargos: ["gravel", "canned_beef"],
                      unique_companies: ["lkwlog", "stokes", "transinet"],
                      job_type_breakdown: { cargo: 1, quick: 1 },
                      brand_distance_km: { Mercedes: 362, Scania: 1482 },
                      cargo_category_coverage: { bulk: 1, refrigerated: 1 },
                    },
                    achievement_groups: [
                      {
                        id: "base_game",
                        name: "Base Game",
                        steam_group_id: 0,
                        steam_dlc_app_id: null,
                      },
                      {
                        id: "special_transport",
                        name: "Special Transport Expansion Achievements",
                        steam_group_id: 2,
                        steam_dlc_app_id: 558245,
                      },
                      {
                        id: "iceland",
                        name: "Iceland Map Expansion Achievements",
                        steam_group_id: 14,
                        steam_dlc_app_id: 3796990,
                      },
                    ],
                    achievements: [
                      {
                        id: "experience_beats_all",
                        group: "base_game",
                        display_name: "Experience Beats All",
                        description:
                          "Complete deliveries with all trailer types.",
                        status: "in_progress",
                        progress: {
                          current: 1,
                          target: 8,
                          unit: "categories",
                        },
                        evidence: [
                          {
                            label: "Bulk cargo",
                            value: "gravel, +103 more",
                            items: ["gravel", "sand"],
                            complete: true,
                          },
                        ],
                      },
                      {
                        id: "test_drive_limited",
                        group: "base_game",
                        display_name: "Test Drive Limited (5 Truck Brands)",
                        description:
                          "Drive 999 km during jobs with each of at least 5 truck brands, only your owned trucks count",
                        status: "in_progress",
                        progress: { current: 1, target: 5, unit: "brands" },
                        evidence: [
                          {
                            label: "Scania",
                            value: "1003 km",
                            complete: true,
                          },
                          { label: "MAN", value: "500 km", complete: false },
                        ],
                      },
                      {
                        id: "long_hauler",
                        group: "base_game",
                        display_name: "Long Hauler",
                        description:
                          "Complete a delivery that was greater than 2,000 km.",
                        status: "complete",
                        progress: {
                          current: 2201,
                          target: 2000,
                          unit: "km",
                        },
                        evidence: [
                          {
                            label: "Longest delivery",
                            value: "gravel, 2201 km, 125000 EUR",
                            complete: true,
                          },
                        ],
                      },
                      {
                        id: "property_magnate",
                        group: "base_game",
                        steam_api_name: "garage_in_every_city",
                        display_name: "Property Magnate",
                        description: "Own at least 30 garages",
                        status: "not_tracked",
                        progress: { current: 0, target: 0, unit: "" },
                        evidence: [
                          {
                            label: "Not tracked yet",
                            value:
                              "Needs owned garage data from the save, which the tracker does not read yet",
                            complete: false,
                          },
                        ],
                      },
                      {
                        id: "size_matters",
                        group: "special_transport",
                        steam_api_name: "st_first_cargo",
                        display_name: "Size Matters",
                        description:
                          "Deliver first oversize cargo (requires Special Transport DLC)",
                        status: "complete",
                        progress: {
                          current: 2,
                          target: 1,
                          unit: "oversize deliveries",
                        },
                        evidence: [
                          {
                            label: "Oversize cargoes",
                            value: "silo, pilot_boat",
                            items: ["pilot_boat", "silo"],
                            complete: true,
                          },
                        ],
                      },
                    ],
                  }),
                },
              });
            } else {
              this.dispatchMessage({
                data: {
                  type: "analysis-error",
                  message:
                    "Structured BSII analysis requires a binary BSII file",
                },
              });
            }
          }
        }, 10);
      }

      dispatchMessage(event) {
        this.onmessage?.(event);
        this.messageListeners.forEach((handler) => handler(event));
      }

      addEventListener(type, handler) {
        if (type === "message") {
          this.messageListeners.add(handler);
        }
      }

      removeEventListener(type, handler) {
        if (type === "message") {
          this.messageListeners.delete(handler);
        }
      }

      terminate() {}
    },
  };
});

beforeEach(() => {
  window.location.hash = "";
});

test("App can render", () => {
  render(<App />);

  expect(screen.getByTestId("file-upload")).toBeInTheDocument();

  const downloadButton = screen.getByTestId("file-download");
  expect(downloadButton).toBeInTheDocument();
  expect(downloadButton).toHaveAttribute("href", "#");

  const textArea = screen.getByTestId("file-display");
  expect(textArea).toBeInTheDocument();
  expect(textArea).toHaveAttribute("readonly");
  expect(textArea).toHaveValue("");
});

test("App can decode", async () => {
  window.location.hash = "#decode";
  render(<App />);
  const fileUploadBox = screen.getByTestId("file-upload");
  const file = new File(["SiiN"], "test.sii");
  await userEvent.upload(fileUploadBox, file);

  const textArea = screen.getByTestId("file-display");
  await waitFor(() => {
    expect(textArea).toHaveValue("SiiN");

    const downloadButton = screen.getByTestId("file-download");
    expect(downloadButton).not.toHaveAttribute("href", "#");
  });
});

test("App can display error", async () => {
  window.location.hash = "#decode";
  render(<App />);
  const fileUploadBox = screen.getByTestId("file-upload");
  const file = new File(["invalid"], "test.sii");
  await userEvent.upload(fileUploadBox, file);

  const textArea = screen.getByTestId("file-display");
  await waitFor(() => {
    expect(textArea).toHaveDisplayValue(/Error:/);

    const downloadButton = screen.getByTestId("file-download");
    expect(downloadButton).toHaveAttribute("href", "#");
  });
});

test("Tracker can render analysis", async () => {
  window.location.hash = "#tracker";
  render(<App />);

  const fileUploadBox = screen.getByTestId("tracker-file-upload");
  const file = new File(["BSII"], "save.sii");
  await userEvent.upload(fileUploadBox, file);

  await waitFor(() => {
    expect(screen.getByTestId("tracker-results")).toBeInTheDocument();
  });
  expect(screen.getByText("Experience Beats All")).toBeInTheDocument();
  expect(screen.getByText("1 / 8 categories")).toBeInTheDocument();
  await userEvent.click(
    screen.getByRole("button", { name: "View 2 delivered cargoes" }),
  );
  expect(
    within(screen.getByRole("dialog")).getByRole("heading", {
      name: "Delivered cargoes",
    }),
  ).toBeInTheDocument();
  expect(screen.getByRole("dialog")).toHaveTextContent("2 delivered cargoes");
  expect(screen.getByRole("dialog")).toHaveTextContent("gravel");
  expect(screen.getByRole("dialog")).toHaveTextContent("sand");
  expect(screen.getByRole("dialog")).not.toHaveTextContent("+103 more");
  await userEvent.click(screen.getByRole("button", { name: "Close" }));
  expect(
    screen.queryByRole("button", { name: /View 3/ }),
  ).not.toBeInTheDocument();
  expect(screen.getByText("gravel, 2201 km, 125000 EUR")).toBeInTheDocument();
  const deliveries = screen.getByText("Deliveries").closest(".metric");
  expect(deliveries).not.toBeNull();
  expect(within(deliveries).getByText("2")).toBeInTheDocument();
  const achievementsMetric = screen
    .getByText("Achievements")
    .closest(".metric");
  expect(within(achievementsMetric).getByText("2 / 5")).toBeInTheDocument();
  const untracked = screen.getByText("Property Magnate").closest("article");
  expect(within(untracked).getByText("Not tracked")).toBeInTheDocument();
  expect(within(untracked).queryByRole("progressbar")).not.toBeInTheDocument();
  expect(
    within(untracked).getByText(/Needs owned garage data/),
  ).toBeInTheDocument();
});

test("Tracker groups achievements by set and expands them on click", async () => {
  window.location.hash = "#tracker";
  render(<App />);

  await userEvent.upload(
    screen.getByTestId("tracker-file-upload"),
    new File(["BSII"], "save.sii"),
  );
  await waitFor(() => {
    expect(screen.getByTestId("tracker-results")).toBeInTheDocument();
  });

  expect(screen.queryByTestId("tracker-stale-decoder")).not.toBeInTheDocument();
  const groups = screen.getAllByTestId("achievement-group");
  // Sets without achievements in the analysis are not rendered.
  expect(groups).toHaveLength(2);

  const baseGame = screen.getByRole("button", { name: /Base Game/ });
  expect(baseGame).toHaveAttribute("aria-expanded", "true");
  expect(baseGame).toHaveTextContent("1 / 4 complete · 3 tracked");
  expect(within(groups[0]).getByText("Achievements (4)")).toBeInTheDocument();

  const specialTransport = screen.getByRole("button", {
    name: /Special Transport Expansion Achievements/,
  });
  expect(specialTransport).toHaveAttribute("aria-expanded", "false");
  expect(specialTransport).toHaveTextContent("1 / 1 complete");
  expect(screen.queryByText("Size Matters")).not.toBeInTheDocument();

  await userEvent.click(specialTransport);
  expect(specialTransport).toHaveAttribute("aria-expanded", "true");
  const sizeMatters = screen.getByText("Size Matters").closest("article");
  await userEvent.click(
    within(sizeMatters).getByRole("button", {
      name: "View 2 delivered oversize cargoes",
    }),
  );
  expect(screen.getByRole("dialog")).toHaveTextContent("pilot_boat");
  await userEvent.click(screen.getByRole("button", { name: "Close" }));

  await userEvent.click(baseGame);
  expect(baseGame).toHaveAttribute("aria-expanded", "false");
  expect(screen.queryByText("Long Hauler")).not.toBeInTheDocument();
});

test("Tracker filters achievements by status across all sets", async () => {
  window.location.hash = "#tracker";
  render(<App />);

  await userEvent.upload(
    screen.getByTestId("tracker-file-upload"),
    new File(["BSII"], "save.sii"),
  );
  await waitFor(() => {
    expect(screen.getByTestId("tracker-results")).toBeInTheDocument();
  });

  const filters = screen.getByRole("group", {
    name: "Filter achievements by status",
  });
  expect(
    within(filters).getByRole("button", { name: "All (5)" }),
  ).toHaveAttribute("aria-pressed", "true");
  expect(
    within(filters).getByRole("button", { name: "In progress (2)" }),
  ).toBeInTheDocument();

  // Complete: matches in both sets, and every matching set opens.
  await userEvent.click(
    within(filters).getByRole("button", { name: "Complete (2)" }),
  );
  expect(screen.getAllByTestId("achievement-group")).toHaveLength(2);
  expect(screen.getByText("Long Hauler")).toBeInTheDocument();
  expect(screen.getByText("Size Matters")).toBeInTheDocument();
  expect(screen.queryByText("Experience Beats All")).not.toBeInTheDocument();
  expect(screen.queryByText("Property Magnate")).not.toBeInTheDocument();
  expect(screen.getByText("Achievements (1 of 4)")).toBeInTheDocument();

  // Not tracked: the Special Transport set has none, so it is hidden.
  await userEvent.click(
    within(filters).getByRole("button", { name: "Not tracked (1)" }),
  );
  expect(screen.getAllByTestId("achievement-group")).toHaveLength(1);
  expect(screen.getByText("Property Magnate")).toBeInTheDocument();
  expect(screen.queryByText("Long Hauler")).not.toBeInTheDocument();

  await userEvent.click(
    within(filters).getByRole("button", { name: "All (5)" }),
  );
  expect(screen.getByText("Long Hauler")).toBeInTheDocument();
  expect(screen.queryByText("Size Matters")).not.toBeInTheDocument();
});

test("Test Drive Limited highlights brands that reached 999 km", async () => {
  window.location.hash = "#tracker";
  render(<App />);

  await userEvent.upload(
    screen.getByTestId("tracker-file-upload"),
    new File(["BSII"], "save.sii"),
  );
  await waitFor(() => {
    expect(screen.getByTestId("tracker-results")).toBeInTheDocument();
  });

  const testDrive = screen
    .getByText("Test Drive Limited (5 Truck Brands)")
    .closest("article");
  const scania = within(testDrive).getByText("Scania").closest("li");
  const man = within(testDrive).getByText("MAN").closest("li");
  expect(scania).toHaveClass("evidence-complete");
  expect(within(scania).getByLabelText("reached")).toBeInTheDocument();
  expect(within(scania).getByText("1003 km")).toBeInTheDocument();
  expect(man).not.toHaveClass("evidence-complete");
  expect(within(man).getByText("500 km")).toBeInTheDocument();

  // Other cards do not highlight their completed evidence rows.
  const longHauler = screen.getByText("Long Hauler").closest("article");
  expect(longHauler.querySelector(".evidence-complete")).toBeNull();
});

test("Tracker warns when the decoder predates achievement sets", async () => {
  window.location.hash = "#tracker";
  render(<App />);

  await userEvent.upload(
    screen.getByTestId("tracker-file-upload"),
    new File(["BSII-old"], "save.sii"),
  );
  await waitFor(() => {
    expect(screen.getByTestId("tracker-results")).toBeInTheDocument();
  });

  expect(screen.getByTestId("tracker-stale-decoder")).toHaveTextContent(
    /older than this page/,
  );
  // Everything falls back into one set.
  expect(screen.getAllByTestId("achievement-group")).toHaveLength(1);
});

test("Tracker can display unsupported-format error", async () => {
  window.location.hash = "#tracker";
  render(<App />);

  const fileUploadBox = screen.getByTestId("tracker-file-upload");
  const file = new File(["SiiN"], "save.sii");
  await userEvent.upload(fileUploadBox, file);

  await waitFor(() => {
    expect(screen.getByTestId("tracker-error")).toHaveTextContent(
      /Structured BSII analysis/,
    );
  });
});

test("Tracker can display worker error", async () => {
  window.location.hash = "#tracker";
  render(<App />);

  const fileUploadBox = screen.getByTestId("tracker-file-upload");
  const file = new File(["invalid"], "save.sii");
  await userEvent.upload(fileUploadBox, file);

  await waitFor(() => {
    expect(screen.getByTestId("tracker-error")).toHaveTextContent(/Error:/);
  });
});

test("Decode errors do not overwrite tracker state after both tabs are used", async () => {
  window.location.hash = "#tracker";
  render(<App />);

  const trackerUpload = screen.getByTestId("tracker-file-upload");
  await userEvent.upload(trackerUpload, new File(["BSII"], "save.sii"));

  await waitFor(() => {
    expect(screen.getByTestId("tracker-results")).toBeInTheDocument();
  });

  await userEvent.click(screen.getByRole("button", { name: "Decode" }));
  await userEvent.upload(
    screen.getByTestId("file-upload"),
    new File(["invalid"], "bad.sii"),
  );

  await waitFor(() => {
    expect(screen.getByTestId("file-display")).toHaveDisplayValue(/Error:/);
  });

  await userEvent.click(screen.getByRole("button", { name: "ETS2 Tracker" }));

  expect(screen.getByTestId("tracker-results")).toBeInTheDocument();
  expect(screen.queryByTestId("tracker-error")).not.toBeInTheDocument();
});

test("Tracker errors do not render in decode output after both tabs are used", async () => {
  window.location.hash = "#decode";
  render(<App />);

  await userEvent.upload(
    screen.getByTestId("file-upload"),
    new File(["SiiN"], "test.sii"),
  );

  await waitFor(() => {
    expect(screen.getByTestId("file-display")).toHaveDisplayValue("SiiN");
  });

  await userEvent.click(screen.getByRole("button", { name: "ETS2 Tracker" }));
  await userEvent.upload(
    screen.getByTestId("tracker-file-upload"),
    new File(["invalid"], "save.sii"),
  );
  await userEvent.click(screen.getByRole("button", { name: "Decode" }));

  await new Promise((resolve) => setTimeout(resolve, 30));

  expect(screen.getByTestId("file-display")).not.toHaveDisplayValue(/Error:/);
});
