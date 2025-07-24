import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

interface CommandRow {
  Packet: {
    label: string;
  };
  Info: {
    label: string;
    status: string;
  };
  Timeout: {
    label: string;
  };
  Status: {
    label: string;
  };
}

export default function CommandPoolExplorer() {
  const [tableData, setTableData] = useState<CommandRow[]>([]);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        console.log("try")
        // The backend returns an array of objects matching the CommandRow shape.
        // We coerce the type here for clarity; you can add runtime validation if desired.
        const data = ((await invoke("miwear_debug_get_commandpool_json_table")) as CommandRow[]);
        console.log(`settabledata: ${JSON.stringify(data)}`)
        setTableData(Array.isArray(data) ? data : []);
      } catch (error) {
        console.error("Failed to fetch command pool:", error);
      }
    }, 10);

    // Clean‑up interval on unmount
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="p-4">
      <h2 className="text-xl font-semibold mb-4">Command Pool Explorer</h2>
      <div className="overflow-x-auto rounded-2xl shadow">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider">Packet</th>
              <th className="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider">Info</th>
              <th className="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider">Kind</th>
              <th className="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider">Timeout</th>
              <th className="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider">Status</th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {tableData.length === 0 && (
              <tr>
                <td colSpan={5} className="text-center py-4 text-gray-400">
                  No commands pending
                </td>
              </tr>
            )}
            {tableData.map((row, idx) => (
              <tr key={idx} className="hover:bg-gray-100">
                <td className="px-6 py-4 whitespace-nowrap">{row.Packet.label}</td>
                <td className="px-6 py-4 whitespace-nowrap">{row.Info.label}</td>
                <td className="px-6 py-4 whitespace-nowrap">{row.Info.status}</td>
                <td className="px-6 py-4 whitespace-nowrap">{row.Timeout.label}</td>
                <td className="px-6 py-4 whitespace-nowrap">{row.Status.label}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}