export const insertZero = async () => {
    const requestBody = {
        key: new Array(32).fill(0), 
        value: 0
    };
    try {
        const response = await fetch('http://localhost:3000/post', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(requestBody)
        });
        
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        console.log('Successfully posted value');
    } catch (error) {
        console.error('Error posting value:', error);
    }
}

export const encrypt = async (value: bigint, ciphertext: any) => {
    const request = {
        key: ciphertext,
        value: Number(value)
    };
    try {
        const response = await fetch('http://localhost:3000/post', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(request)
        });
        
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        console.log('Successfully posted value');
    } catch (error) {
        console.error('Error posting value:', error);
    }
}

export const transfer = async (senderCiphertext: any, recipientCiphertext: any, transferCiphertext: any) => {
    const request = {
        sender_key: senderCiphertext,
        recipient_key: recipientCiphertext,
        transfer_value: transferCiphertext
    };

    try {
        console.log('Sending transfer request:', JSON.stringify(request, null, 2));
        const response = await fetch('http://localhost:3000/transfer', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(request)
        });

        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(`HTTP error! status: ${response.status}, body: ${errorText}`);
        }

        console.log('Successfully processed transfer');
    } catch (error) {
        console.error('Detailed transfer error:', error);
        throw error;
    }
};

export const fhe8add = async (lhs_key: any, rhs_key: any, result_key: any) => {
    const request = {
        lhs_key: lhs_key,
        rhs_key: rhs_key,
        result_key: result_key,
    };

    try {
        console.log('Sending fhe8 add request:', JSON.stringify(request, null, 2));
        const response = await fetch('http://localhost:3000/fhe8add', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(request)
        });

        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(`HTTP error! status: ${response.status}, body: ${errorText}`);
        }

        console.log('Successfully processed fhe 8 add');
    } catch (error) {
        console.error('Detailed fhe 8 add error:', error);
        throw error;
    }
};
