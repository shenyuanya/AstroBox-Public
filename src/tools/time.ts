export function timeToRead(time: number): string {
    const seconds = Math.floor(time / 1000);
    if (seconds < 60) {
        return `${seconds}秒`;
    }
    const minutes = Math.floor(time / 1000 / 60);
    if (minutes < 60) {
        return `${minutes}分钟`;
    }
    const hours = Math.floor(time / 1000 / 60 / 60);
    if (hours < 24) {
        return `${hours}小时`;
    }
    const days = Math.floor(time / 1000 / 60 / 60 / 24);
    return `${days}天`;
}