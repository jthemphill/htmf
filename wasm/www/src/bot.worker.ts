const ctx: Worker = self as any;

const Bot = (await import("./Bot")).default;
const bot = new Bot(ctx.postMessage);
ctx.onmessage = (event: MessageEvent) => {
  bot.onMessage(event.data);
};
bot.init();

export { };