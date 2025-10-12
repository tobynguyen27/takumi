import { useEffect, useRef, useState } from "react";
import type { PanelGroupProps } from "react-resizable-panels";
import { useSearchParams } from "react-router";
import defaultTemplate from "~/playground/default?raw";
import { compressCode, decompressCode } from "~/playground/share";
import TakumiWorker from "~/playground/worker?worker";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "../ui/resizable";
import { ComponentEditor } from "./editor";

const mobileViewportWidth = 640;

function useDirection() {
  const [direction, setDirection] =
    useState<PanelGroupProps["direction"]>("horizontal");

  const resize = () => {
    setDirection(
      window.innerWidth < mobileViewportWidth ? "vertical" : "horizontal",
    );
  };

  useEffect(() => {
    resize();

    addEventListener("resize", resize);

    return () => removeEventListener("resize", resize);
  });

  return direction;
}

export default function Playground() {
  const [code, setCode] = useState<string>();
  const [rendered, setRendered] = useState<string>();
  const [isReady, setIsReady] = useState(false);

  const workerRef = useRef<Worker | undefined>(undefined);
  const direction = useDirection();
  const [searchParams, setSearchParams] = useSearchParams();

  const codeQuery = searchParams.get("code");

  useEffect(() => {
    if (codeQuery) decompressCode(codeQuery).then(setCode);
    else setCode(defaultTemplate);
  }, [codeQuery]);

  useEffect(() => {
    if (!code) return;

    if (code === defaultTemplate) {
      return setSearchParams((prev) => {
        prev.delete("code");

        return prev;
      });
    }

    compressCode(code).then((base64) => {
      setSearchParams((prev) => {
        prev.set("code", base64);

        return prev;
      });
    });
  }, [code, setSearchParams]);

  useEffect(() => {
    const worker = new TakumiWorker();

    worker.onmessage = (event: MessageEvent) => {
      if (event.data.type === "ready") {
        setIsReady(true);
      } else if (event.data.type === "render_complete") {
        setRendered(event.data.dataUrl);
      } else if (event.data.type === "render_error") {
        console.error("Worker render error:", event.data.error);
      }
    };

    workerRef.current = worker;

    return () => {
      worker.terminate();
      workerRef.current = undefined;
      setIsReady(false);
    };
  }, []);

  useEffect(() => {
    if (isReady) {
      workerRef.current?.postMessage({
        type: "render",
        code,
      });
    }
  }, [isReady, code]);

  return (
    <div className="h-[calc(100dvh-3.5rem)]">
      <ResizablePanelGroup direction={direction}>
        <ResizablePanel defaultSize={50}>
          {code && <ComponentEditor code={code} setCode={setCode} />}
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel defaultSize={50}>
          <ResizablePanelGroup direction="vertical">
            <ResizablePanel
              defaultSize={50}
              className="flex justify-center items-center"
            >
              {rendered && (
                <img
                  className="w-full h-full object-contain"
                  src={rendered}
                  alt="Takumi rendered result"
                />
              )}
            </ResizablePanel>
            <ResizableHandle withHandle />
            <ResizablePanel defaultSize={50}>
              <div className="h-full overflow-y-auto p-4">
                <p className="text-lg py-2 font-medium">Viewport</p>
                <p>1200 x 630</p>
              </div>
            </ResizablePanel>
          </ResizablePanelGroup>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}
