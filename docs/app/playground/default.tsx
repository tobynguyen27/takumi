export default function Image() {
  return (
    <div
      tw="w-full h-full justify-center bg-black items-center"
      style={{
        backgroundImage:
          "radial-gradient(circle at 25px 25px, lightgray 2%, transparent 0%), radial-gradient(circle at 75px 75px, lightgray 2%, transparent 0%)",
        backgroundSize: "100px 100px",
      }}
    >
      <div tw="justify-center items-center flex flex-col text-white">
        <h1 tw="font-semibold text-6xl block whitespace-pre mt-0">
          Welcome to <span tw="text-[#ff3535]">Takumi </span>
          Playground 👋
        </h1>
        <span tw="opacity-75 text-4xl font-[Geist_Mono]">
          You can try out and experiment with Takumi here.
        </span>
      </div>
    </div>
  );
}

const devicePixelRatio = 1.0;

export const options: PlaygroundOptions = {
  width: 1200 * devicePixelRatio,
  height: 630 * devicePixelRatio,
  format: "png",
  devicePixelRatio,
};
