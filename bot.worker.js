const Bot = (await import("./Bot")).default;
const ctx = self;
const bot = new Bot(ctx.postMessage);
ctx.onmessage = (event) => {
    bot.onMessage(event.data);
};
bot.init();
export {};
//# sourceMappingURL=bot.worker.js.map