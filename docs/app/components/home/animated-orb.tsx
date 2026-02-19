export function AnimatedOrb() {
  return (
    <div className="absolute inset-0 pointer-events-none overflow-hidden">
      <div className="absolute top-[35%] left-1/2 w-[600px] h-[600px] bg-[radial-gradient(circle_at_40%_40%,var(--primary)_0%,rgba(255,53,53,0.3)_40%,transparent_70%)] blur-[100px] opacity-15 animate-orb-float" />
      <div className="absolute top-[25%] left-[55%] w-[400px] h-[400px] bg-[radial-gradient(circle_at_60%_60%,#ffa944_0%,rgba(255,169,68,0.2)_40%,transparent_70%)] blur-[100px] opacity-10 animate-orb-float-alt" />
    </div>
  );
}
