import { Banner } from "@douyinfe/semi-ui";
import { useTranslation } from "react-i18next";
import { databases } from "../../../data/databases";
import { useEffect, useState } from "react";
import { diagramService } from "../../../services/diagramService";

export default function Open({ selectedDiagramId, setSelectedDiagramId }) {
  const [diagrams, setDiagrams] = useState(null);
  const { t } = useTranslation();

  useEffect(() => {
    let mounted = true;
    (async () => {
      try {
        const list = await diagramService.list();
        if (!mounted) return;
        setDiagrams(
          list.map((d) => ({
            ...d,
            lastModified: d.lastModified ? new Date(d.lastModified) : new Date(),
          }))
        );
      } catch (e) {
        console.error(e);
        if (mounted) setDiagrams([]);
      }
    })();
    return () => {
      mounted = false;
    };
  }, []);

  const getDiagramSize = (d) => {
    const size = JSON.stringify(d).length;
    let sizeStr;
    if (size >= 1024 && size < 1024 * 1024)
      sizeStr = (size / 1024).toFixed(1) + "KB";
    else if (size >= 1024 * 1024)
      sizeStr = (size / (1024 * 1024)).toFixed(1) + "MB";
    else sizeStr = size + "B";

    return sizeStr;
  };
  return (
    <div>
      {diagrams?.length === 0 ? (
        <Banner
          fullMode={false}
          type="info"
          bordered
          icon={null}
          closeIcon={null}
          description={<div>You have no saved diagrams.</div>}
        />
      ) : (
        <div className="max-h-[360px]">
          <table className="w-full text-left border-separate border-spacing-x-0">
            <thead>
              <tr>
                <th>{t("name")}</th>
                <th>{t("last_modified")}</th>
                <th>{t("size")}</th>
                <th>{t("type")}</th>
              </tr>
            </thead>
            <tbody>
              {diagrams?.map((d) => {
                return (
                  <tr
                    key={d.id}
                    className={`${
                      selectedDiagramId === d.id
                        ? "bg-blue-300/30"
                        : "hover-1"
                    }`}
                    onClick={() => {
                      setSelectedDiagramId(d.id);
                    }}
                  >
                    <td className="py-1">
                      <i className="bi bi-file-earmark-text text-[16px] me-1 opacity-60" />
                      {d.name}
                    </td>
                    <td className="py-1">
                      {d.lastModified.toLocaleDateString() +
                        " " +
                        d.lastModified.toLocaleTimeString()}
                    </td>
                    <td className="py-1">{getDiagramSize(d)}</td>
                    <td className="py-1">
                      {databases[d.database].name ?? "Generic"}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
