UPDATE prizes SET
    stock = stock + $1
WHERE prizeName = $2;
