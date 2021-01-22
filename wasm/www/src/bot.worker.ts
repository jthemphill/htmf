const Bot = (await import("./Bot")).default;

const ctx: Worker = self as any;

const bot = new Bot(ctx.postMessage);
ctx.onmessage = (event: MessageEvent) => {
  bot.onMessage(event.data);
};
bot.init();

export { };