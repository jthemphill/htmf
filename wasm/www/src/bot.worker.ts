const Bot = (await import("./Bot")).default;

const bot = new Bot(postMessage);
onmessage = (event: MessageEvent) => {
  bot.onMessage(event.data);
};
bot.init();

export { };